use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard};

use futures::{Sink, SinkExt, Stream};

use crate::client::Client;

use crate::models::gateway::message::{ClientMsg, ServerMsg};

mod conn;
mod error;
mod socket;

pub use conn::GatewayConnection;
pub use error::GatewayError;
pub use socket::GatewaySocket;

pub struct Gateway {
    client: Client,
    conn: GatewayConnection,
}

impl Gateway {
    pub fn new(client: &Client) -> Self {
        Gateway {
            client: client.clone(),
            conn: GatewayConnection::new(client.clone()),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }
}
