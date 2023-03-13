#![allow(clippy::type_complexity)]

use std::sync::Arc;

use crate::{
    client::Client,
    gateway::{GatewayConnection, GatewayConnectionControl},
};

use futures::{stream::SplitSink, SinkExt, StreamExt};
use tokio::sync::mpsc;

pub mod cmd;

mod ctx;
mod error;
mod util;

pub use ctx::StandardContext;
pub use error::{StandardError, StandardErrorExt};

use self::ctx::InternalEventHandlers;

use super::{ClientMsg, DynamicServerMsgHandlers, ServerMsgHandlers};

/// Dynamic [`ServerMsgHandlers`](super::ServerMsgHandlers) suitable for simpler bot applications
pub type StandardDynamicHandler<S, E> = DynamicServerMsgHandlers<StandardContext, Result<(), E>, S>;

/// Simple [`Standard`] with a [`StandardError`]
pub type SimpleStandard<H> = Standard<H, StandardError>;

/// Runs whenever an error occurs
type ErrorCb<H, E> = Arc<dyn Fn(E, StandardContext, &H) + Send + Sync + 'static>;
/// Runs once after first gateway connection is established
type StartCb<H, E> = Box<dyn FnOnce(StandardContext, &mut H) -> Result<(), E>>;

pub struct Standard<H, E: StandardErrorExt = StandardError> {
    state: ctx::InternalEventHandlers<H>,
    ctx: StandardContext,
    gateway: GatewayConnection,
    on_error: Option<ErrorCb<H, E>>,
    on_start: Option<StartCb<H, E>>,
    rx: mpsc::UnboundedReceiver<ctx::StandardResponse>,
}

impl<E: StandardErrorExt> Standard<StandardDynamicHandler<(), E>, E> {
    pub fn new(client: Client) -> Self {
        Self::new_with_state(client, ())
    }
}

impl<S, E: StandardErrorExt> Standard<StandardDynamicHandler<S, E>, E>
where
    S: Send + Sync + 'static,
{
    pub fn new_with_state(client: Client, state: S) -> Self {
        Self::new_with_handlers(
            client,
            StandardDynamicHandler::new_raw_with_state(state, Box::new(|_, _, _| Box::pin(util::ZSTOkFut::new()))),
        )
    }
}

impl<H: 'static, E: StandardErrorExt> Standard<H, E>
where
    H: ServerMsgHandlers<StandardContext, Result<(), E>>,
{
    pub fn new_with_handlers(client: Client, state: H) -> Self {
        let (ctx, rx) = StandardContext::new(client.clone());

        Standard {
            state: ctx::InternalEventHandlers::new(state),
            gateway: GatewayConnection::new(client),
            ctx,
            rx,
            on_error: None,
            on_start: None,
        }
    }

    /// Setup a callback for any errors that occur during the connection lifetime
    pub fn on_error<F>(&mut self, cb: F) -> &mut Self
    where
        F: Fn(E, StandardContext, &H) + Send + Sync + 'static,
    {
        self.on_error = Some(Arc::new(cb));
        self
    }

    /// Setup a callback to run once after the first gateway connection is established,
    /// but before any kind of authentication is made.
    ///
    /// If the gateway reconnects, this will not be rerun.
    pub fn on_start<F>(&mut self, cb: F) -> &mut Self
    where
        F: FnOnce(StandardContext, &mut H) -> Result<(), E> + 'static,
    {
        self.on_start = Some(Box::new(cb));
        self
    }

    pub fn handlers(&mut self) -> &mut H {
        &mut self.state.user
    }

    pub fn ctx(&self) -> &StandardContext {
        &self.ctx
    }

    pub fn gateway_control(&self) -> Arc<GatewayConnectionControl> {
        self.gateway.control()
    }

    pub async fn run(self) -> Result<(), E> {
        let Standard {
            mut state,
            ctx,
            mut gateway,
            on_error,
            on_start,
            rx,
        } = self;

        // connect to gateway first, split streams
        let (gw_tx, mut gw_rx) = {
            gateway.connect().await?;
            gateway.split()
        };

        if let Some(on_start) = on_start {
            if let Err(e) = on_start(ctx.clone(), &mut state.user) {
                if let Some(ref err_cb) = on_error {
                    err_cb(e, ctx.clone(), &state.user);
                }
            }
        }

        let (kill, mut alive) = tokio::sync::oneshot::channel::<()>();

        // state has been finalized, wrap it in Arc to share among server/client tasks
        let state = Arc::new(state);

        // start running client msg task
        let client_task = tokio::spawn(run_client(
            rx,
            gw_tx,
            state.clone(),
            ctx.clone(),
            on_error.clone(),
            kill,
        ));

        // begin listening for events on current task
        loop {
            let event = tokio::select! {
                biased;
                _ = &mut alive => break,
                event = gw_rx.next() => match event {
                    Some(event) => event,
                    None => break,
                },
            };

            let res = match event {
                Err(e) => Err(e.into()),
                Ok(msg) => state.dispatch(ctx.clone(), msg).await,
            };

            if let Err(e) = res {
                if let Some(ref err_cb) = on_error {
                    err_cb(e, ctx.clone(), &state.user);
                }
            }
        }

        let _ = client_task.await;

        Ok(())
    }
}

async fn run_client<H, E: StandardErrorExt>(
    mut rx: mpsc::UnboundedReceiver<ctx::StandardResponse>,
    mut gw_tx: SplitSink<GatewayConnection, ClientMsg>,
    state: Arc<InternalEventHandlers<H>>,
    ctx: StandardContext,
    on_error: Option<ErrorCb<H, E>>,
    kill: tokio::sync::oneshot::Sender<()>,
) {
    while let Some(resp) = rx.recv().await {
        match resp {
            ctx::StandardResponse::Message(msg) => {
                if let Err(e) = gw_tx.send(msg).await {
                    if let Some(ref err_cb) = on_error {
                        err_cb(e.into(), ctx.clone(), &state.user);
                    }
                }
            }
            ctx::StandardResponse::Close => {
                rx.close();
                let _ = kill.send(());
                return;
            }
        }
    }
}
