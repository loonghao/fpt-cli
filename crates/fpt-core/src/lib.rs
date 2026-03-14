pub mod error;
pub mod io;
pub mod model;
pub mod spec;

pub use error::{AppError, ErrorCode, ErrorEnvelope, Result};
pub use io::read_json_input;
pub use model::{OutputFormat, RiskLevel};
pub use spec::CommandSpec;

