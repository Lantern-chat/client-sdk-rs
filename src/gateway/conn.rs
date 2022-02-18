use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};

use futures::{FutureExt, Sink, SinkExt, Stream};

use crate::client::Client;
use crate::models::gateway::message::{ClientMsg, ServerMsg};

use super::{GatewayError, GatewaySocket};

/// Gateway connection that provides automatic reconnect
/// functionality as part of the [Sink]/[Stream] APIs.
///
/// However, it does not automatically perform the [Hello](ServerMsg::Hello)/[Identify](ClientMsg::Identify) handshake.
///
/// Upon reconnecting the underlying websocket, the server will send
/// a [Hello](ServerMsg::Hello) event to initiate the handshake.
pub struct GatewayConnection {
    client: Client,
    closed: AtomicBool,
    connecting: Option<Pin<Box<dyn Future<Output = Result<GatewaySocket, GatewayError>>>>>,
    socket: Option<GatewaySocket>,
}

impl GatewayConnection {
    pub fn new(client: Client) -> GatewayConnection {
        GatewayConnection {
            client,
            closed: AtomicBool::new(false),
            connecting: None,
            socket: None,
        }
    }

    /// Manually connect the gateway websocket
    pub async fn connect(&mut self) -> Result<(), GatewayError> {
        futures::future::poll_fn(move |cx| self.poll_project_socket(cx).map_ok(|_| ())).await
    }

    /// Acquire a pinned projection of the socket, or poll the connecting future.
    fn poll_project_socket(
        &mut self,
        cx: &mut Context,
    ) -> Poll<Result<Pin<&mut GatewaySocket>, GatewayError>> {
        // fast path, project socket
        if let Some(ref mut socket) = self.socket {
            return Poll::Ready(Ok(Pin::new(socket)));
        }

        self.poll_project_socket_cold(cx)
    }

    /// Poll the connecting future to acquire a new socket over time
    #[inline(never)]
    fn poll_project_socket_cold(
        &mut self,
        cx: &mut Context,
    ) -> Poll<Result<Pin<&mut GatewaySocket>, GatewayError>> {
        // if there is no connecting future, set one up
        if self.connecting.is_none() {
            if self.closed.load(Ordering::SeqCst) {
                return Poll::Ready(Err(GatewayError::Disconnected));
            }

            self.connecting = Some(GatewaySocket::connect(self.client.clone()).boxed());
        }

        match self.connecting {
            Some(ref mut connecting) => match connecting.poll_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                Poll::Ready(Ok(socket)) => {
                    self.socket = Some(socket);
                    self.connecting = None;

                    // just assigned, project
                    Poll::Ready(Ok(Pin::new(self.socket.as_mut().unwrap())))
                }
            },
            // just checked/assigned, so this path is impossible
            None => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    fn reconnect(&mut self, cx: &mut Context) -> Poll<Result<Pin<&mut GatewaySocket>, GatewayError>> {
        self.socket = None;
        self.poll_project_socket_cold(cx)
    }
}

impl Stream for GatewayConnection {
    type Item = Result<ServerMsg, GatewayError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let socket = match self.poll_project_socket(cx) {
            Poll::Ready(Ok(socket)) => socket,
            Poll::Ready(Err(e)) => return Poll::Ready(Some(Err(e))),
            Poll::Pending => return Poll::Pending,
        };

        let res = futures::ready!(socket.poll_next(cx));

        match res {
            None => {}
            Some(Err(ref e)) if e.is_close() => {}
            _ => return Poll::Ready(res),
        }

        assert!(self.reconnect(cx).is_pending());
        Poll::Pending
    }
}

impl Sink<ClientMsg> for GatewayConnection {
    type Error = GatewayError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), GatewayError>> {
        match self.poll_project_socket(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Err(e)) if e.is_close() => {
                assert!(self.reconnect(cx).is_pending());
                Poll::Pending
            }
            res => res.map_ok(|_| ()),
        }
    }

    #[inline]
    fn start_send(mut self: Pin<&mut Self>, item: ClientMsg) -> Result<(), GatewayError> {
        match self.socket {
            Some(ref mut socket) => socket.start_send_unpin(item),

            // `start_send` doesn't poll or have a context, so there is no way to initiate the reconnect
            None => Err(GatewayError::Disconnected),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), GatewayError>> {
        let socket = match futures::ready!(self.poll_project_socket(cx)) {
            Ok(socket) => socket,
            err => return Poll::Ready(err).map_ok(|_| ()),
        };

        socket.poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Result<(), GatewayError>> {
        // ensure it won't reconnect automatically
        self.closed.store(true, Ordering::SeqCst);

        let socket = match futures::ready!(self.poll_project_socket(cx)) {
            // kind of done its job at this point...
            Err(GatewayError::Disconnected) => return Poll::Ready(Ok(())),
            Ok(socket) => socket,
            Err(e) => return Poll::Ready(Err(e)),
        };

        let res = socket.poll_close(cx);

        // success, fully close internal socket by dropping it
        if let Poll::Ready(Ok(())) = res {
            self.get_mut().socket = None;
        }

        res
    }
}
