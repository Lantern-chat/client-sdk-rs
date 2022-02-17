use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{Sink, Stream};
use tokio_tungstenite::tungstenite::Message as WsMessage;

type WebSocket = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

use crate::client::Client;
use crate::driver::Encoding;
use crate::models::gateway::message::{ClientMsg, ServerMsg};

use super::GatewayError;

pin_project_lite::pin_project! {
    /// Raw WebSocket adapter that handles encoding and decoding of messages
    pub struct GatewaySocket {
        #[pin]
        ws: WebSocket,
        encoding: Encoding,
        compress: bool,
    }
}

impl GatewaySocket {
    pub async fn connect(client: Client) -> Result<Self, GatewayError> {
        unimplemented!()
    }

    fn encode(&self, msg: ClientMsg) -> Result<WsMessage, GatewayError> {
        unimplemented!()
    }

    fn decode(&self, msg: WsMessage) -> Result<ServerMsg, GatewayError> {
        if msg.is_close() {
            return Err(GatewayError::Disconnected);
        }

        unimplemented!()
    }
}

impl Sink<ClientMsg> for GatewaySocket {
    type Error = GatewayError;

    #[inline]
    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), GatewayError>> {
        self.project().ws.poll_ready(cx).map_err(GatewayError::from)
    }

    #[inline]
    fn start_send(self: Pin<&mut Self>, msg: ClientMsg) -> Result<(), GatewayError> {
        let item = self.encode(msg)?;
        self.project().ws.start_send(item).map_err(GatewayError::from)
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), GatewayError>> {
        self.project().ws.poll_flush(cx).map_err(GatewayError::from)
    }

    #[inline]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), GatewayError>> {
        self.project().ws.poll_close(cx).map_err(GatewayError::from)
    }
}

impl Stream for GatewaySocket {
    type Item = Result<ServerMsg, GatewayError>;

    #[inline]
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        let this = self.as_mut().project();

        Poll::Ready(match this.ws.poll_next(cx) {
            Poll::Ready(None) => None,
            Poll::Ready(Some(Ok(msg))) => Some(self.decode(msg)),
            Poll::Ready(Some(Err(e))) => Some(Err(e.into())),
            Poll::Pending => return Poll::Pending,
        })
    }
}
