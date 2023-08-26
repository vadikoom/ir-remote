use std::{
    io,
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};

use tokio::{
    net::UdpSocket,
    select,
    sync::{mpsc, watch},
};
use tracing::{info, instrument};

pub use handle::ServerHandle;
pub use runner::ServerRunner;

mod handle;
mod runner;
mod transport;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[allow(dead_code)]
    #[error("Not Implemented")]
    NotImplemented,

    #[error("IO Error occurred in server: {0}")]
    Io(#[from] io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Timeout while waiting for MPC response")]
    MpcResponseTimeout,

    #[error("IR did not acknowledge the request in time")]
    IRAckTimeout,

    #[error("{0}")]
    MpcGenericError(&'static str),

    #[error("Can not send message when IR is offline")]
    CanNotSendWhenOffline
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
enum MpcRequest {
    SendRcRequest(&'static [u32]),
}

#[derive(Debug)]
enum MpcResponse {
    SendRcResponse(Result<()>),
}

pub async fn new(bind_address: Ipv4Addr, bind_port: u16) -> Result<(ServerRunner, ServerHandle)> {
    let (tx, rx) = mpsc::channel(1);
    let (tx_online, rx_online) = watch::channel(false);
    let addr = SocketAddrV4::new(bind_address, bind_port);

    info!("starting server on {:?}", addr);
    let socket = UdpSocket::bind(addr).await?;

    Ok((ServerRunner::new(rx, socket, tx_online), ServerHandle::new(tx, rx_online)))
}
