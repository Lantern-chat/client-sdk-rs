use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{Sink, Stream};
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::Message as WsMessage;

type WebSocket = tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

use crate::driver::{Driver, Encoding};
use crate::models::gateway::message::{ClientMsg, ServerMsg};

use super::error::GatewayErrorCode;
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
    pub async fn connect(driver: Driver) -> Result<Self, GatewayError> {
        // TODO: Setup a circuit breaker to run this in a loop until it succeeds
        let (ws, _) = tokio_tungstenite::connect_async(format!(
            "ws{}/api/v1/gateway?compress=true&encoding={}",
            &driver.uri[4..],
            match driver.encoding {
                Encoding::Json => "json",
                #[cfg(feature = "cbor")]
                Encoding::CBOR => "cbor",
            }
        ))
        .await?;

        Ok(GatewaySocket {
            ws,
            encoding: driver.encoding,
            compress: true,
        })
    }

    fn encode(&self, msg: ClientMsg) -> Result<WsMessage, GatewayError> {
        let mut body = Vec::new();

        match self.encoding {
            Encoding::Json => serde_json::to_writer(&mut body, &msg)?,
            #[cfg(feature = "cbor")]
            Encoding::CBOR => ciborium::ser::into_writer(&msg, &mut body)?,
        }

        if self.compress {
            body = miniz_oxide::deflate::compress_to_vec_zlib(&body, 9);
        }

        Ok(WsMessage::Binary(body))
    }

    fn decode(&self, msg: WsMessage) -> Result<ServerMsg, GatewayError> {
        match &msg {
            WsMessage::Close(None) => return Err(GatewayError::Disconnected),
            WsMessage::Close(Some(msg)) => {
                return Err(match msg.code {
                    CloseCode::Library(code) => {
                        use num_traits::FromPrimitive;

                        GatewayError::CloseError(match GatewayErrorCode::from_u16(code) {
                            Some(code) => code,
                            None => GatewayErrorCode::UnknownError,
                        })
                    }
                    _ => GatewayError::Disconnected,
                });
            }
            _ => {}
        }

        let mut body = msg.into_data();

        if self.compress {
            body = match miniz_oxide::inflate::decompress_to_vec_zlib(&body) {
                Ok(body) => body,
                Err(_) => return Err(GatewayError::CompressionError),
            };
        }

        Ok(match self.encoding {
            Encoding::Json => serde_json::from_slice(&body)?,
            #[cfg(feature = "cbor")]
            Encoding::CBOR => ciborium::de::from_reader(&body[..])?,
        })
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
