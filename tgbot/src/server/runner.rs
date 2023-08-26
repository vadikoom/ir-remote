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
}

impl ServerRunner {
    pub(super) fn new(
        rx: mpsc::Receiver<(MpcRequest, oneshot::Sender<MpcResponse>)>,
        socket: UdpSocket,
        online: watch::Sender<bool>,
    ) -> Self {
        Self { rx, socket, state: ServerState::new(online) }
    }

    #[instrument(name = "ServerRunner::run", skip_all)]
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting server IO loop");
        let Self { rx, socket, state } = self;
        let (mut transport_input, mut transport_output)  = transport::transport(&*socket);
        let mut last_from = None;

        loop {
            let now = Instant::now();
            let timeout = state.next_timeout(now);
            let cmd = state
                .command_to_send_now(now)
                .map(|c| Command { data: c,  sequence: state.next_seq.0 - 1})
                .map(|c| last_from.map(|f| (c, f)))
                .flatten();

            select! {
                _ = async { tokio::time::sleep(timeout.unwrap()).await }, if timeout.is_some() => {
                    state.timeout(Instant::now());
                }

                _ = async { transport_output.send(cmd.unwrap()).await }, if cmd.is_some() => {
                    state.successful_send();
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
        let d = self.clients.front().map(|(_, i, _)| i.clone());

        let option = [a, b, c, d]
            .into_iter()
            .filter_map(|a| a)
            .reduce(|a, b| a.min(b))
            .map(|at| at.max(now) - now);

        option
    }

    pub fn command_to_send_now(&self, now: Instant) -> Option<&'static [u32]> {
        let should_send_now = self.next_send_attempt_at.map_or(false, |i| i <= now);

        if should_send_now {
            self.current_cmd
        } else {
            None
        }
    }

    pub fn successful_send(&mut self) {
        info!("command sent successfully");

        self.next_send_attempt_at.as_mut().map(|x| *x += DURATION_BETWEEN_RETRY);
    }

    #[instrument(skip_all)]
    pub fn timeout(&mut self, now: Instant) {
        info!("timeout event");
        if self.online_until.map_or(false, |t| t <= now) {
            // remove device did not send any message until timeout
            // don't care if there are no receivers
            warn!("online_channel changes to false");
            let _ = self.online_channel.send(false);
            self.online_until = None
        }

        if self.attempt_sending_cmd_until.map_or(false, |t| t <= now) {
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
                    let _ = tx.send(MpcResponse::SendRcResponse(Err(Error::CanNotSendWhenOffline)));
                    return;
                }

                // we don't care if we had other command sent before, just taking this one
                // and starting sending it from scratch

                let now = Instant::now();
                let ack = self.next_seq;

                info!("Got new remote command, assigning ack={:?}", ack);

                self.next_seq.0 += 1;
                self.current_cmd = Some(cmd);
                self.attempt_sending_cmd_until = Some(now + DURATION_ALL_RETRIES);
                self.next_send_attempt_at = Some(now);
                self.clients.push_back((ack, now + DURATION_ALL_RETRIES, tx));
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
            info!("IR acknowledged sec={}, but we have next value = {}. setting next to = {}", sec.0, self.next_seq.0, sec.0 + 1);
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
