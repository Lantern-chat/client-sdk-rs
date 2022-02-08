use std::fmt;

use http::Method;

pub(crate) mod sealed {
    pub trait Sealed {}
}

use crate::models::Permission;

pub trait Command: sealed::Sealed + serde::Serialize {
    type Result;

    const METHOD: Method;
    const PERMS: Permission;

    /// Serialize/format the URI path (with query)
    fn format_path<W: fmt::Write>(&self, w: W) -> fmt::Result;
}
