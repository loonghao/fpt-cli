#![allow(clippy::result_large_err)]

pub mod error;
pub mod io;
pub mod model;
pub mod spec;

pub use error::{AppError, ErrorCode, ErrorEnvelope, Result};
pub use io::read_json_input;
pub use model::{OutputFormat, RiskLevel};
pub use spec::CommandSpec;

/// Transport label for the REST API surface.
///
/// Shared across crates so that error envelopes and transport plans use a
/// single canonical constant instead of duplicating the `"rest"` string.
pub const TRANSPORT_REST: &str = "rest";
