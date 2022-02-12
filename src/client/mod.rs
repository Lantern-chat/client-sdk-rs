use std::sync::Arc;

use arc_swap::{ArcSwap, ArcSwapOption};
use headers::authorization::{Authorization, Bearer};

use crate::{
    driver::{Driver, Encoding},
    models::SmolToken,
};

mod error;
pub use error::ClientError;

mod file;

struct ClientInner {
    inner: reqwest::Client,
    auth: ArcSwapOption<Authorization<Bearer>>,
    uri: ArcSwap<String>,
    preferred_encoding: ArcSwap<Encoding>,
}

#[derive(Clone)]
pub struct Client(Arc<ClientInner>);

impl ClientInner {
    /// (Cheaply) Constructs a new Driver instance for a request
    fn driver(&self) -> Driver {
        Driver {
            inner: self.inner.clone(),
            auth: self.auth.load_full(),
            uri: self.uri.load_full(),
            encoding: **self.preferred_encoding.load(),
        }
    }
}

impl Client {
    pub fn set_token(&self, token: Option<SmolToken>) -> Result<(), ClientError> {
        self.0.auth.store(match token {
            None => None,
            Some(token) => match Authorization::bearer(&token) {
                Ok(auth) => Some(Arc::new(auth)),
                Err(_) => return Err(ClientError::InvalidBearerToken),
            },
        });

        Ok(())
    }

    pub fn set_auth(&self, auth: Option<Authorization<Bearer>>) {
        self.0.auth.store(auth.map(Arc::new));
    }

    pub fn set_preferred_encoding(&self, encoding: Encoding) {
        self.0.preferred_encoding.store(Arc::new(encoding));
    }

    /// Constructs a [Driver] instance with the current configuration. Changes to the Client configuration
    /// will not be reflected in the created Driver, and a new one must be constructed.
    ///
    /// This operation is decently cheap. (A few atomic loads)
    #[inline]
    pub fn driver(&self) -> Driver {
        self.0.driver()
    }

    /// Consumes a [Driver] to initialize the Client
    pub fn from_driver(driver: Driver) -> Self {
        Client(Arc::new(ClientInner {
            inner: driver.inner,
            auth: ArcSwapOption::new(driver.auth),
            uri: ArcSwap::new(driver.uri),
            preferred_encoding: ArcSwap::new(Arc::new(driver.encoding)),
        }))
    }
}
