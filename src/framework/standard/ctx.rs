use std::sync::{
    atomic::{AtomicU32, Ordering::SeqCst},
    Arc,
};

use crate::{
    client::Client,
    driver::Driver,
    framework::{ServerMsg, ServerMsgHandlers},
    models::{gateway::message::ClientMsg, *},
};

use tokio::sync::mpsc::{self, error::SendError};

#[derive(Debug)]
pub enum StandardResponse {
    Message(ClientMsg),
    Close,
}

struct StandardContextInner {
    tx: mpsc::UnboundedSender<StandardResponse>,
    client: Client,
}

#[derive(Clone)]
pub struct StandardContext(Arc<StandardContextInner>);

impl StandardContext {
    pub(super) fn new(client: Client) -> (Self, mpsc::UnboundedReceiver<StandardResponse>) {
        let (tx, rx) = mpsc::unbounded_channel();

        (StandardContext(Arc::new(StandardContextInner { client, tx })), rx)
    }

    fn inner(&self) -> &StandardContextInner {
        &self.0
    }

    pub fn client(&self) -> &Client {
        &self.inner().client
    }

    pub fn driver(&self) -> Driver {
        self.client().driver()
    }

    pub fn close(&self) -> bool {
        self.inner().tx.send(StandardResponse::Close).is_ok()
    }

    pub fn send(&self, msg: ClientMsg) -> Result<(), SendError<ClientMsg>> {
        match self.inner().tx.send(StandardResponse::Message(msg)) {
            Ok(()) => Ok(()),
            Err(SendError(StandardResponse::Message(msg))) => Err(SendError(msg)),
            Err(_) => unreachable!(),
        }
    }

    pub fn set_presence(&self, presence: UserPresence) -> Result<(), SendError<UserPresence>> {
        match self.send(ClientMsg::new_set_presence(commands::SetPresence { presence })) {
            Ok(()) => Ok(()),
            Err(SendError(ClientMsg::SetPresence(payload))) => Err(SendError(payload.inner.presence)),
            Err(_) => unreachable!(),
        }
    }
}

use tokio::sync::Notify;

pub struct InternalEventHandlers<H> {
    pub user: H,
    heartbeat: Arc<Notify>,
    interval: AtomicU32,
}

impl<H> InternalEventHandlers<H> {
    pub fn new(state: H) -> Self {
        InternalEventHandlers {
            user: state,
            heartbeat: Default::default(),
            interval: AtomicU32::new(45_000),
        }
    }

    #[allow(clippy::needless_return)] // ugh
    fn setup_new_heartbeat(&self, ctx: StandardContext) {
        let hb = self.heartbeat.clone();
        let interval = self.interval.load(SeqCst);

        hb.notify_waiters();
        tokio::spawn(async move {
            let duration = tokio::time::Duration::from_millis(interval as u64);

            tokio::time::sleep(duration).await;

            if ctx.send(ClientMsg::new_heartbeat()).is_err() {
                return;
            }

            // once the heartbeat is sent, the server will respond with ack and
            // trigger the killswitch to cancel this sleep, avoiding `close()`
            // but if the server does not respond, close will be called

            let mut sleep = std::pin::pin!(tokio::time::sleep(duration));

            tokio::select! {
                biased;
                _ = hb.notified() => return,
                _ = &mut sleep => ctx.close(),
            };
        });
    }
}

use crate::models::events::*;
impl<H, E> ServerMsgHandlers<StandardContext, Result<(), E>> for InternalEventHandlers<H>
where
    H: ServerMsgHandlers<StandardContext, Result<(), E>>,
{
    #[inline(always)]
    async fn fallback(&self, ctx: StandardContext, msg: ServerMsg) -> Result<(), E> {
        self.user.dispatch(ctx, msg).await
    }

    async fn hello(&self, ctx: StandardContext, inner: Hello) -> Result<(), E> {
        self.interval.store(inner.heartbeat_interval, SeqCst);

        self.setup_new_heartbeat(ctx.clone());

        if let Some(auth) = ctx.client().auth() {
            let _ = ctx.send(ClientMsg::new_identify(commands::Identify {
                auth,
                intent: Intent::all(),
            }));
        }

        self.user.hello(ctx, inner).await
    }

    async fn heartbeat_ack(&self, ctx: StandardContext) -> Result<(), E> {
        self.setup_new_heartbeat(ctx.clone());

        self.user.heartbeat_ack(ctx).await
    }

    //async fn ready(&self, ctx: StandardContext, ready: Box<Ready>) -> Result<(), E> {
    //    Ok(())
    //}
}
