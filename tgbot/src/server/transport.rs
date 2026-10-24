use std::net::SocketAddr;

use bytes::BytesMut;
use futures::{Sink, Stream};
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
use tokio_util::codec::{Decoder, Encoder};
use tokio_util::udp::UdpFramed;

#[derive(Serialize, Debug)]
pub struct Command {
    pub data: &'static [u32],
    pub sequence: u32,
}

#[derive(Deserialize, Debug)]
pub struct Status {
    pub last_command_sequence_number: u32,
}

pub struct Codec(super::crypto::Crypto);
impl Decoder for Codec {
    type Item = Status;
    type Error = super::Error;

    fn decode(&mut self, src: &mut BytesMut) -> super::Result<Option<Self::Item>> {
        if src.is_empty() {
            return Ok(None);
        }

        let plaintext = self.0.open(super::crypto::STATUS, &src.split())?;
        let status: Status = serde_json::from_slice(&plaintext)?;
        if status.last_command_sequence_number == u32::MAX {
            return Err(super::Error::Security("Sequence number exhausted"));
        }
        Ok(Some(status))
    }
}

impl Encoder<Command> for Codec {
    type Error = super::Error;

    fn encode(
        &mut self,
        item: Command,
        dst: &mut BytesMut,
    ) -> std::result::Result<(), Self::Error> {
        if item.sequence == u32::MAX
            || item.data.is_empty()
            || item.data.len() > 300
            || item
                .data
                .iter()
                .any(|&value| value == 0 || value > u16::MAX as u32)
        {
            return Err(super::Error::Security("Invalid IR command"));
        }
        let packet = self
            .0
            .seal(super::crypto::COMMAND, &serde_json::to_vec(&item)?)?;
        dst.extend_from_slice(&packet);
        Ok(())
    }
}

pub fn transport(
    socket: &UdpSocket,
    crypto: super::crypto::Crypto,
) -> (
    impl Stream<Item = super::Result<(Status, SocketAddr)>> + '_,
    impl Sink<(Command, SocketAddr), Error = super::Error> + '_,
) {
    let a = UdpFramed::new(socket, Codec(crypto.clone()));
    let b = UdpFramed::new(socket, Codec(crypto));
    (a, b)
}
