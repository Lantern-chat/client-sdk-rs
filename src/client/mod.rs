use std::sync::Arc;

use arc_swap::{ArcSwap, ArcSwapOption};

use crate::{
    driver::{generic_client, Driver, DriverError, Encoding},
    models::AuthToken,
};

mod error;
pub use error::ClientError;

mod file;

struct ClientInner {
    inner: reqwest::Client,
    auth: ArcSwapOption<(AuthToken, reqwest::header::HeaderValue)>,
    uri: Arc<str>,
    preferred_encoding: ArcSwap<Encoding>,
}

#[must_use = "Client does nothing on its own."]
#[derive(Clone)]
pub struct Client(Arc<ClientInner>);

impl ClientInner {
    /// (Cheaply) Constructs a new Driver instance for a request
    fn driver(&self) -> Driver {
        Driver {
            inner: self.inner.clone(),
            auth: self.auth.load_full(),
            uri: self.uri.clone(),
            encoding: **self.preferred_encoding.load(),
        }
    }
}

impl Client {
    pub fn new(uri: &str) -> Result<Self, ClientError> {
        Ok(Self::from_client(generic_client().build()?, uri))
    }

    pub fn from_client(client: reqwest::Client, uri: &str) -> Self {
        Client(Arc::new(ClientInner {
            inner: client,
            auth: ArcSwapOption::empty(),
            uri: Arc::from(uri),
            preferred_encoding: ArcSwap::from_pointee(Encoding::JSON),
        }))
    }

    pub fn set_auth(&self, token: Option<AuthToken>) -> Result<(), ClientError> {
        self.0.auth.store(match token {
            None => None,
            Some(token) => Some(Arc::new((
                token,
                match token.headervalue() {
                    Ok(header) => header,
                    Err(e) => return Err(ClientError::DriverError(DriverError::from(e))),
                },
            ))),
        });

        Ok(())
    }

    #[must_use]
    pub fn auth(&self) -> Option<AuthToken> {
        self.0.auth.load().as_ref().map(|auth| auth.0)
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
}
