use core::error::Error;

use crate::{client::ClientError, driver::DriverError, gateway::GatewayError};

#[derive(Debug, thiserror::Error)]
pub enum StandardError {
    #[error(transparent)]
    ClientError(#[from] ClientError),

    #[error(transparent)]
    DriverError(#[from] DriverError),

    #[error(transparent)]
    GatewayError(#[from] GatewayError),
}

/// Required properties for custom error types,
/// must be able to handle [`ClientError`], [`DriverError`], and [`GatewayError`] errors
pub trait StandardErrorExt: 'static + Error + From<ClientError> + From<DriverError> + From<GatewayError> {}
impl<T> StandardErrorExt for T where T: 'static + Error + From<ClientError> + From<DriverError> + From<GatewayError> {}
