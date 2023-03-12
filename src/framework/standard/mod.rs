use std::sync::Arc;

use crate::{
    client::{Client, ClientError},
    driver::{Driver, DriverError},
};

use super::{DynamicServerMsgHandlers, ServerMsg, ServerMsgHandlers};

#[derive(Debug, thiserror::Error)]
pub enum StandardError {
    #[error(transparent)]
    ClientError(#[from] ClientError),

    #[error(transparent)]
    DriverError(#[from] DriverError),
}

pub type StandardResult<T> = Result<T, StandardError>;

pub type StandardDynamicHandler<S> = DynamicServerMsgHandlers<StandardContext, StandardResult<()>, S>;

struct InternalEventHandlers<H> {
    user: H,
}

use std::{future::Future, pin::Pin};

#[async_trait::async_trait]
impl<H> ServerMsgHandlers<StandardContext, StandardResult<()>> for InternalEventHandlers<H>
where
    H: ServerMsgHandlers<StandardContext, StandardResult<()>>,
{
    #[inline(always)]
    fn fallback<'life0, 'async_trait>(
        &'life0 self,
        ctx: StandardContext,
        msg: ServerMsg,
    ) -> Pin<Box<dyn Future<Output = StandardResult<()>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        self.user.dispatch(ctx, msg)
    }
}

struct StandardContextInner {
    client: Client,
}

#[derive(Clone)]
pub struct StandardContext(Arc<StandardContextInner>);

impl StandardContext {
    pub fn client(&self) -> &Client {
        &self.0.client
    }

    pub fn driver(&self) -> Driver {
        self.client().driver()
    }
}

pub struct Standard<H> {
    state: InternalEventHandlers<H>,
    ctx: StandardContext,
    on_error: Option<Box<dyn Fn(StandardError, &H) + Send + 'static>>,
}

impl Standard<StandardDynamicHandler<()>> {
    pub fn new(client: Client) -> Self {
        Self::new_with_state(client, ())
    }
}

impl<S> Standard<StandardDynamicHandler<S>>
where
    S: Send + Sync + 'static,
{
    pub fn new_with_state(client: Client, state: S) -> Self {
        Self::new_with_handlers(
            client,
            StandardDynamicHandler::new_with_state(state, |_, _, _| async { Ok(()) }),
        )
    }
}

impl<H> Standard<H>
where
    H: ServerMsgHandlers<StandardContext, StandardResult<()>>,
{
    pub fn new_with_handlers(client: Client, state: H) -> Self {
        Standard {
            state: InternalEventHandlers { user: state },
            ctx: StandardContext(Arc::new(StandardContextInner { client })),
            on_error: None,
        }
    }

    pub fn on_error<F>(&mut self, cb: F) -> &mut Self
    where
        F: Fn(StandardError, &H) + Send + 'static,
    {
        self.on_error = Some(Box::new(cb));
        self
    }

    pub fn handlers(&mut self) -> &mut H {
        &mut self.state.user
    }

    pub fn ctx(&self) -> &StandardContext {
        &self.ctx
    }

    async fn dispatch_event(self, msg: ServerMsg) {
        if let Err(e) = self.state.dispatch(self.ctx.clone(), msg).await {
            if let Some(ref err_cb) = self.on_error {
                err_cb(e, &self.state.user);
            }
        }
    }

    pub async fn run(self) -> StandardResult<()> {
        let gateway = crate::gateway::GatewayConnection::new(self.ctx().client().clone());

        // TODO

        Ok(())
    }
}
