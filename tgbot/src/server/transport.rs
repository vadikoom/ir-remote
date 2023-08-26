use std::net::SocketAddr;

use bytes::{Buf, BufMut, BytesMut};
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

pub struct Codec;
impl Decoder for Codec {
    type Item = Status;
    type Error = super::Error;

    fn decode(&mut self, src: &mut BytesMut) -> super::Result<Option<Self::Item>> {
        if src.is_empty() {
            return Ok(None);
        }

        Ok(Some(serde_json::from_reader(src.split().reader())?))
    }
}

impl Encoder<Command> for Codec {
    type Error = super::Error;

    fn encode(&mut self, item: Command, dst: &mut BytesMut) -> std::result::Result<(), Self::Error> {
        serde_json::to_writer(dst.writer(), &item)?;
        Ok(())
    }
}


pub fn transport(socket: &UdpSocket) -> (
    impl Stream<Item=super::Result<(Status, SocketAddr)>> + '_,
    impl Sink<(Command, SocketAddr), Error=super::Error> + '_
) {
    let a = UdpFramed::new(socket, Codec);
    let b = UdpFramed::new(socket, Codec);
    (a, b)
}
