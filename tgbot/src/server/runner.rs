use std::{collections::VecDeque, time::Instant};

use futures::prelude::*;
use tokio::sync::{mpsc, oneshot, watch};
use tracing::{error, warn};

use crate::server::transport::Command;

use super::*;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Debug)]
struct SequenceNumber(u32);

pub struct ServerRunner {
    rx: mpsc::Receiver<(MpcRequest, oneshot::Sender<MpcResponse>)>,
    socket: UdpSocket,
    state: ServerState,
    crypto: crypto::Crypto,
}

impl ServerRunner {
    pub(super) fn new(
        rx: mpsc::Receiver<(MpcRequest, oneshot::Sender<MpcResponse>)>,
        socket: UdpSocket,
        online: watch::Sender<bool>,
        crypto: crypto::Crypto,
    ) -> Self {
        Self {
            rx,
            socket,
            state: ServerState::new(online),
            crypto,
        }
    }

    #[instrument(name = "ServerRunner::run", skip_all)]
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting server IO loop");
        let Self {
            rx,
            socket,
            state,
            crypto,
        } = self;
        let (mut transport_input, mut transport_output) =
            transport::transport(&*socket, crypto.clone());
        let mut last_from = None;

        loop {
            let now = Instant::now();
            let timeout = state.next_timeout(now);
            let cmd = state
                .command_to_send_now(now)
                .map(|c| Command {
                    data: c,
                    sequence: state.next_seq.0 - 1,
                })
                .zip(last_from);

            select! {
                _ = async { tokio::time::sleep(timeout.unwrap()).await }, if timeout.is_some() => {
                    state.timeout(Instant::now());
                }

                result = async { transport_output.send(cmd.unwrap()).await }, if cmd.is_some() => {
                    match result {
                        Ok(()) => info!("command sent successfully"),
                        Err(error) => {
                            warn!(?error, "Failed to send command; retrying until acknowledgement deadline");
                            // UdpFramed retains a failed datagram until its buffer is replaced.
                            transport_output = transport::transport(&*socket, crypto.clone()).1;
                        }
                    }
                    state.send_attempted();
                }

                status = transport_input.next() => {
                    match status {
                        None => {
                            error!("Empty message, socket was closed");
                            break;
                        }

                        Some(Ok((status, from))) => {
                            info!("got status: {:?} from {:?}", status, from);
                            last_from = Some(from);
                            state.ack_from_remote(SequenceNumber(status.last_command_sequence_number));
                        }

                        Some(Err(e)) => {
                            warn!("got error from remote channel: {:?}", e);
                        }
                    }
                }

                cmd = rx.recv() => {
                    match cmd {
                        Some((cmd, response)) => {
                            state.new_command(cmd, response);
                        }
                        None => {
                            info!("Exiting server because there is no more available client channels");
                            break;
                        }
                    }
                }
            }
        }

        info!("Exiting server IO loop");
        Ok(())
    }
}

const DURATION_ALL_RETRIES: Duration = Duration::from_secs(4);
const DURATION_BETWEEN_RETRY: Duration = Duration::from_secs(1);
const DURATION_ONLINE: Duration = Duration::from_secs(40);

struct ServerState {
    online_until: Option<Instant>,
    next_seq: SequenceNumber,
    last_ack: SequenceNumber,
    online_channel: watch::Sender<bool>,

    current_cmd: Option<&'static [u32]>,
    next_send_attempt_at: Option<Instant>,
    attempt_sending_cmd_until: Option<Instant>,

    clients: VecDeque<(SequenceNumber, Instant, oneshot::Sender<MpcResponse>)>,
}

impl ServerState {
    pub fn new(online_channel: watch::Sender<bool>) -> Self {
        ServerState {
            online_until: None,
            online_channel,
            next_seq: SequenceNumber(1),
            last_ack: SequenceNumber(0),
            current_cmd: None,
            attempt_sending_cmd_until: None,
            next_send_attempt_at: None,
            clients: VecDeque::new(),
        }
    }

    pub fn next_timeout(&self, now: Instant) -> Option<Duration> {
        let a = self.online_until;
        let b = self.next_send_attempt_at;
        let c = self.attempt_sending_cmd_until;
        let d = self.clients.front().map(|(_, i, _)| *i);

        [a, b, c, d]
            .into_iter()
            .flatten()
            .min()
            .map(|at| at.max(now) - now)
    }

    pub fn command_to_send_now(&self, now: Instant) -> Option<&'static [u32]> {
        let should_send_now = self.next_send_attempt_at.is_some_and(|i| i <= now);

        if should_send_now {
            self.current_cmd
        } else {
            None
        }
    }

    pub fn send_attempted(&mut self) {
        self.next_send_attempt_at = Some(Instant::now() + DURATION_BETWEEN_RETRY);
    }

    #[instrument(skip_all)]
    pub fn timeout(&mut self, now: Instant) {
        info!("timeout event");
        if self.online_until.is_some_and(|t| t <= now) {
            // remove device did not send any message until timeout
            // don't care if there are no receivers
            warn!("online_channel changes to false");
            let _ = self.online_channel.send(false);
            self.online_until = None
        }

        if self.attempt_sending_cmd_until.is_some_and(|t| t <= now) {
            warn!(
                "remove device did not confirm current command until timeout. giving up sending \
                 the command"
            );
            self.attempt_sending_cmd_until = None;
            self.next_send_attempt_at = None;
            self.current_cmd = None;
        }

        self.respond_to_clients(now);
    }

    #[instrument(skip_all)]
    pub fn new_command(&mut self, cmd: MpcRequest, tx: oneshot::Sender<MpcResponse>) {
        match cmd {
            MpcRequest::SendRcRequest(cmd) => {
                // we don't accept new commands when offline
                if self.online_until.is_none() {
                    warn!("Rejecting new command because offline");
                    // don't care of rx is dropped
                    let _ = tx.send(MpcResponse::SendRcResponse(Err(
                        Error::CanNotSendWhenOffline,
                    )));
                    return;
                }

                for (_, _, response) in self.clients.drain(..) {
                    let _ =
                        response.send(MpcResponse::SendRcResponse(Err(Error::CommandSuperseded)));
                }

                let now = Instant::now();
                let ack = self.next_seq;

                info!("Got new remote command, assigning ack={:?}", ack);

                self.next_seq.0 += 1;
                self.current_cmd = Some(cmd);
                self.attempt_sending_cmd_until = Some(now + DURATION_ALL_RETRIES);
                self.next_send_attempt_at = Some(now);
                self.clients
                    .push_back((ack, now + DURATION_ALL_RETRIES, tx));
            }
        }
    }

    #[instrument(skip_all)]
    pub fn ack_from_remote(&mut self, sec: SequenceNumber) {
        let now = Instant::now();

        if self.online_until.is_none() {
            info!("remote device status changes to online");
            // we don't care if there are no receivers
            let _ = self.online_channel.send(true);
        }

        self.online_until = Some(now + DURATION_ONLINE);
        self.last_ack = sec;

        if self.last_ack.0 + 1 == self.next_seq.0 {
            info!(
                "got the ack for the current command we sending, stop sending it, ack={:?}",
                self.last_ack
            );
            self.current_cmd = None;
            self.attempt_sending_cmd_until = None;
            self.next_send_attempt_at = None;
        }

        if self.next_seq <= self.last_ack {
            info!(
                "IR acknowledged sec={}, but we have next value = {}. setting next to = {}",
                sec.0,
                self.next_seq.0,
                sec.0 + 1
            );
            self.next_seq.0 = sec.0 + 1;
        }

        self.respond_to_clients(now);
    }

    // responds to clients with either timeout or success
    fn respond_to_clients(&mut self, now: Instant) {
        loop {
            match self.clients.pop_front() {
                None => {
                    break;
                }

                Some((sn, i, tx)) => {
                    let response = if sn <= self.last_ack {
                        info!(
                            "Responding to client with success. client expects ack={:?}, current \
                             ack={:?}",
                            sn, self.last_ack
                        );
                        MpcResponse::SendRcResponse(Ok(()))
                    } else if i <= now {
                        info!(
                            "Responding to client with timeout error. client expects ack={:?}, \
                             current ack={:?}",
                            sn, self.last_ack
                        );
                        MpcResponse::SendRcResponse(Err(Error::IRAckTimeout))
                    } else {
                        self.clients.push_front((sn, i, tx));
                        break;
                    };

                    // we don't care if the client did not wait for response long enough
                    let _ = tx.send(response);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn online_state(sequence: u32) -> ServerState {
        let (online, _rx) = watch::channel(false);
        let mut state = ServerState::new(online);
        state.ack_from_remote(SequenceNumber(sequence));
        state
    }

    #[tokio::test]
    async fn superseded_command_is_cancelled_even_when_new_command_is_acknowledged() {
        let mut state = online_state(0);
        let (first_tx, first_rx) = oneshot::channel();
        let (second_tx, second_rx) = oneshot::channel();
        state.new_command(MpcRequest::SendRcRequest(&[111]), first_tx);
        state.new_command(MpcRequest::SendRcRequest(&[222]), second_tx);
        assert!(matches!(
            first_rx.await.unwrap(),
            MpcResponse::SendRcResponse(Err(Error::CommandSuperseded))
        ));
        assert_eq!(state.command_to_send_now(Instant::now()), Some(&[222][..]));
        state.ack_from_remote(SequenceNumber(2));
        assert!(matches!(
            second_rx.await.unwrap(),
            MpcResponse::SendRcResponse(Ok(()))
        ));
        assert!(state.current_cmd.is_none());
    }

    #[tokio::test]
    async fn unacknowledged_command_times_out_and_stops_retrying() {
        let mut state = online_state(0);
        let (tx, rx) = oneshot::channel();
        state.new_command(MpcRequest::SendRcRequest(&[111]), tx);
        state.timeout(Instant::now() + DURATION_ALL_RETRIES);
        assert!(matches!(
            rx.await.unwrap(),
            MpcResponse::SendRcResponse(Err(Error::IRAckTimeout))
        ));
        assert!(state.current_cmd.is_none());
        assert!(state.next_send_attempt_at.is_none());
    }

    #[tokio::test]
    async fn offline_transmitter_rejects_commands() {
        let (online, _rx) = watch::channel(false);
        let mut state = ServerState::new(online);
        let (tx, rx) = oneshot::channel();
        state.new_command(MpcRequest::SendRcRequest(&[111]), tx);
        assert!(matches!(
            rx.await.unwrap(),
            MpcResponse::SendRcResponse(Err(Error::CanNotSendWhenOffline))
        ));
    }

    #[test]
    fn restart_synchronizes_sequence_and_send_attempts_are_spaced() {
        let mut state = online_state(42);
        let (tx, _rx) = oneshot::channel();
        state.new_command(MpcRequest::SendRcRequest(&[111]), tx);
        assert_eq!(state.next_seq, SequenceNumber(44));
        state.send_attempted();
        assert!(state.command_to_send_now(Instant::now()).is_none());
        assert_eq!(
            state.command_to_send_now(Instant::now() + DURATION_BETWEEN_RETRY),
            Some(&[111][..])
        );
    }

    #[tokio::test]
    async fn rejected_oversized_command_does_not_poison_following_commands() {
        let (mut runner, handle) = super::super::new(
            Ipv4Addr::LOCALHOST,
            0,
            crypto::Crypto::from_hex(crypto::TEST_SECRET).unwrap(),
        )
        .await
        .unwrap();
        let addr = runner.socket.local_addr().unwrap();
        let device = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let task = tokio::spawn(async move { runner.run().await });
        device
            .send_to(
                &crypto::Crypto::from_hex(crypto::TEST_SECRET)
                    .unwrap()
                    .seal(crypto::STATUS, br#"{"last_command_sequence_number":0}"#)
                    .unwrap(),
                addr,
            )
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(2), async {
            while !handle.is_online() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        let result = tokio::time::timeout(Duration::from_secs(6), handle.send_rc(&[65535; 20_000]))
            .await
            .unwrap();
        assert!(matches!(result, Err(Error::IRAckTimeout)));

        let call = tokio::spawn(async move { handle.send_rc(&[100, 200]).await });
        let mut buf = [0; 2048];
        let (len, _) = tokio::time::timeout(Duration::from_secs(2), device.recv_from(&mut buf))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(
                &crypto::Crypto::from_hex(crypto::TEST_SECRET)
                    .unwrap()
                    .open(crypto::COMMAND, &buf[..len])
                    .unwrap()
            )
            .unwrap(),
            serde_json::json!({"data": [100, 200], "sequence": 2})
        );
        device
            .send_to(
                &crypto::Crypto::from_hex(crypto::TEST_SECRET)
                    .unwrap()
                    .seal(crypto::STATUS, br#"{"last_command_sequence_number":2}"#)
                    .unwrap(),
                addr,
            )
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(2), call)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        tokio::time::timeout(Duration::from_secs(2), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }

    #[tokio::test]
    async fn udp_round_trip_retries_and_recovers_after_transmitter_reconnect() {
        let (mut runner, handle) = super::super::new(
            Ipv4Addr::LOCALHOST,
            0,
            crypto::Crypto::from_hex(crypto::TEST_SECRET).unwrap(),
        )
        .await
        .unwrap();
        let addr = runner.socket.local_addr().unwrap();
        let device = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let task = tokio::spawn(async move { runner.run().await });
        device.send_to(b"invalid JSON", addr).await.unwrap();
        device
            .send_to(
                &crypto::Crypto::from_hex(crypto::TEST_SECRET)
                    .unwrap()
                    .seal(crypto::STATUS, br#"{"last_command_sequence_number":41}"#)
                    .unwrap(),
                addr,
            )
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(2), async {
            while !handle.is_online() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();

        let attacker = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        attacker
            .send_to(br#"{"last_command_sequence_number":1000}"#, addr)
            .await
            .unwrap();
        let forged = crypto::Crypto::from_hex(&"ff".repeat(32))
            .unwrap()
            .seal(crypto::STATUS, br#"{"last_command_sequence_number":1000}"#)
            .unwrap();
        attacker.send_to(&forged, addr).await.unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        let sender = handle.clone();
        let mut call = tokio::spawn(async move { sender.send_rc(&[100, 200, 300]).await });
        let mut buf = [0; 2048];
        for _ in 0..2 {
            let (len, _) = tokio::time::timeout(Duration::from_secs(2), device.recv_from(&mut buf))
                .await
                .unwrap()
                .unwrap();
            let wire: serde_json::Value = serde_json::from_slice(
                &crypto::Crypto::from_hex(crypto::TEST_SECRET)
                    .unwrap()
                    .open(crypto::COMMAND, &buf[..len])
                    .unwrap(),
            )
            .unwrap();
            assert_eq!(
                wire,
                serde_json::json!({"data": [100, 200, 300], "sequence": 42})
            );
        }
        assert!(tokio::time::timeout(Duration::from_millis(20), &mut call)
            .await
            .is_err());
        device
            .send_to(
                &crypto::Crypto::from_hex(crypto::TEST_SECRET)
                    .unwrap()
                    .seal(crypto::STATUS, br#"{"last_command_sequence_number":42}"#)
                    .unwrap(),
                addr,
            )
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(2), call)
            .await
            .unwrap()
            .unwrap()
            .unwrap();

        let reconnected = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        reconnected
            .send_to(
                &crypto::Crypto::from_hex(crypto::TEST_SECRET)
                    .unwrap()
                    .seal(crypto::STATUS, br#"{"last_command_sequence_number":42}"#)
                    .unwrap(),
                addr,
            )
            .await
            .unwrap();
        let call = tokio::spawn(async move { handle.send_rc(&[400, 500]).await });
        let (len, _) =
            tokio::time::timeout(Duration::from_secs(2), reconnected.recv_from(&mut buf))
                .await
                .unwrap()
                .unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(
                &crypto::Crypto::from_hex(crypto::TEST_SECRET)
                    .unwrap()
                    .open(crypto::COMMAND, &buf[..len])
                    .unwrap()
            )
            .unwrap(),
            serde_json::json!({"data": [400, 500], "sequence": 43})
        );
        reconnected
            .send_to(
                &crypto::Crypto::from_hex(crypto::TEST_SECRET)
                    .unwrap()
                    .seal(crypto::STATUS, br#"{"last_command_sequence_number":43}"#)
                    .unwrap(),
                addr,
            )
            .await
            .unwrap();
        tokio::time::timeout(Duration::from_secs(2), call)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        tokio::time::timeout(Duration::from_secs(2), task)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
    }
}
