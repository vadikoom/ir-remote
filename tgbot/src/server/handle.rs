use tokio::sync::{mpsc, oneshot, watch};

use super::*;

#[derive(Clone)]
pub struct ServerHandle {
    tx: mpsc::Sender<(MpcRequest, oneshot::Sender<MpcResponse>)>,
    online: watch::Receiver<bool>,
}

impl ServerHandle {
    pub(super) fn new(
        tx: mpsc::Sender<(MpcRequest, oneshot::Sender<MpcResponse>)>,
        online: watch::Receiver<bool>,
    ) -> Self {
        Self { tx, online }
    }

    pub async fn send_rc(&self, cmd: &'static [u32]) -> Result<()> {
        let result = select! {
            x = self.send_command(MpcRequest::SendRcRequest(cmd)) => { x? }
            _ = tokio::time::sleep(Duration::from_secs(10)) => { Result::Err(Error::MpcResponseTimeout)? }
        };

        let MpcResponse::SendRcResponse(r) = result;
        r
    }

    pub fn is_online(&self) -> bool {
        self.online.borrow().clone()
    }

    async fn send_command(&self, cmd: MpcRequest) -> Result<MpcResponse> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send((cmd, tx))
            .await
            .map_err(|_| Error::MpcGenericError("failed sending MPC message"))?;

        let result = rx.await.map_err(|_| {
            Error::MpcGenericError("Server dropped channel before sending response")
        })?;

        Ok(result)
    }
}
