use crate::models::*;
use smol_str::SmolStr;

// Implementation notes:
//
// * Commands that provide a body to the request should split the body into its own
//   struct to be used directly within the server, sharing the type and ensuring compatibility.
// * Body structs will also implement Deserialize, so ensure proper default behavior

pub mod room;
