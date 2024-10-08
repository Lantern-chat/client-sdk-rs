use core::num::NonZeroUsize;
use core::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::{future::BoxFuture, FutureExt, Sink, SinkExt, Stream};

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
///
/// Any errors that occur will still be passed through, and must be handled appropriately. Spamming
/// servers will reconnections will lead to rate-limiting and possibly automated bans.
pub struct GatewayConnection {
    client: Client,
    connecting: Option<BoxFuture<'static, Result<GatewaySocket, GatewayError>>>,
    socket: Option<GatewaySocket>,
    control: Arc<GatewayConnectionControl>,
}

pub struct GatewayConnectionControl {
    closed: AtomicBool,
    reconnects: AtomicUsize,
    reconnect_limit: AtomicUsize,
}

impl GatewayConnectionControl {
    /// Resets the connection attempt counter and opens up for new connections.
    pub fn reset(&self) {
        self.reconnects.store(0, Ordering::SeqCst);
        self.closed.store(false, Ordering::SeqCst);
    }

    /// Sets the reconnect limit to a non-zero value. Does not immediately disconnect if
    /// the current reconnection counter is above this.
    pub fn set_reconnect_limit(&self, limit: NonZeroUsize) {
        self.reconnect_limit.store(limit.get(), Ordering::SeqCst);
    }

    /// Prevent reconnecting
    pub fn noreconnect(&self) {
        self.closed.store(true, Ordering::SeqCst);
    }
}

impl GatewayConnection {
    #[must_use]
    pub fn new(client: Client) -> GatewayConnection {
        GatewayConnection {
            client,
            connecting: None,
            socket: None,
            control: Arc::new(GatewayConnectionControl {
                closed: AtomicBool::new(false),
                reconnects: AtomicUsize::new(0),
                reconnect_limit: AtomicUsize::new(20),
            }),
        }
    }

    /// Manually initiate a new connection of the gateway websocket
    ///
    /// This does not handle any responses to server events.
    pub async fn connect(&mut self) -> Result<(), GatewayError> {
        futures::future::poll_fn(move |cx| self.poll_project_socket(cx).map_ok(|_| ())).await
    }

    /// Get a reference to the control structure
    pub fn control(&self) -> Arc<GatewayConnectionControl> {
        self.control.clone()
    }

    /// Acquire a pinned projection of the socket, or poll the connecting future.
    fn poll_project_socket(&mut self, cx: &mut Context<'_>) -> Poll<Result<Pin<&mut GatewaySocket>, GatewayError>> {
        // fast path, project socket
        if let Some(ref mut socket) = self.socket {
            return Poll::Ready(Ok(Pin::new(socket)));
        }

        self.poll_project_socket_cold(cx)
    }

    /// Poll the connecting future to acquire a new socket over time
    #[inline(never)]
    fn poll_project_socket_cold(&mut self, cx: &mut Context<'_>) -> Poll<Result<Pin<&mut GatewaySocket>, GatewayError>> {
        // if there is no connecting future, set one up
        if self.connecting.is_none() {
            if self.control.closed.load(Ordering::SeqCst) {
                return Poll::Ready(Err(GatewayError::Disconnected));
            }

            let limit = self.control.reconnect_limit.load(Ordering::SeqCst);
            if self.control.reconnects.fetch_add(1, Ordering::SeqCst) > limit {
                self.control.closed.store(true, Ordering::SeqCst);

                return Poll::Ready(Err(GatewayError::ReconnectLimitExceeded(limit)));
            }

            self.connecting = Some(GatewaySocket::connect(self.client.driver()).boxed());
        }

        match self.connecting {
            Some(ref mut connecting) => match connecting.poll_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                Poll::Ready(Ok(socket)) => {
                    self.socket = Some(socket);
                    self.connecting = None;

                    Poll::Ready(Ok(match self.socket {
                        // just assigned, project
                        Some(ref mut socket) => Pin::new(socket),
                        None => unsafe { core::hint::unreachable_unchecked() },
                    }))
                }
            },
            // just checked/assigned, so this path is impossible
            None => unsafe { core::hint::unreachable_unchecked() },
        }
    }
}

impl Stream for GatewayConnection {
    type Item = Result<ServerMsg, GatewayError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let res = match self.poll_project_socket(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Ok(socket)) => futures::ready!(socket.poll_next(cx)),
            Poll::Ready(Err(e)) => Some(Err(e)),
        };

        if let None | Some(Err(_)) = res {
            self.socket = None; // drop socket
        }

        Poll::Ready(res)
    }
}

impl Sink<ClientMsg> for GatewayConnection {
    type Error = GatewayError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), GatewayError>> {
        let res = match self.poll_project_socket(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(Ok(socket)) => futures::ready!(socket.poll_ready(cx)),
            Poll::Ready(Err(err)) => Err(err),
        };

        if res.is_err() {
            self.socket = None; // drop socket
        }

        Poll::Ready(res)
    }

    #[inline]
    fn start_send(mut self: Pin<&mut Self>, item: ClientMsg) -> Result<(), GatewayError> {
        match self.socket {
            Some(ref mut socket) => socket.start_send_unpin(item).inspect_err(|_| {
                self.socket = None; // drop socket
            }),
            // `start_send` doesn't poll or have a context, so there is no way to initiate the reconnect
            None => Err(GatewayError::Disconnected),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), GatewayError>> {
        let res = match futures::ready!(self.poll_project_socket(cx)) {
            Ok(socket) => futures::ready!(socket.poll_flush(cx)),
            Err(e) => Err(e),
        };

        if res.is_err() {
            self.socket = None; // drop socket
        }

        Poll::Ready(res)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), GatewayError>> {
        // ensure it won't reconnect automatically
        self.control.closed.store(true, Ordering::SeqCst);

        let res = match futures::ready!(self.poll_project_socket(cx)) {
            Ok(socket) => futures::ready!(socket.poll_close(cx)),
            Err(e) => Err(e),
        };

        self.socket = None; // drop socket

        Poll::Ready(res)
    }
}
