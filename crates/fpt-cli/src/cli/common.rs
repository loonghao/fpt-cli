use clap::ValueEnum;
use fpt_core::OutputFormat;
use fpt_domain::AuthMode;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormatArg {
    Toon,
    Json,
    PrettyJson,
}

impl From<OutputFormatArg> for OutputFormat {
    fn from(value: OutputFormatArg) -> Self {
        match value {
            OutputFormatArg::Toon => OutputFormat::Toon,
            OutputFormatArg::Json => OutputFormat::Json,
            OutputFormatArg::PrettyJson => OutputFormat::PrettyJson,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AuthModeArg {
    Script,
    UserPassword,
    SessionToken,
}

impl From<AuthModeArg> for AuthMode {
    fn from(value: AuthModeArg) -> Self {
        match value {
            AuthModeArg::Script => AuthMode::Script,
            AuthModeArg::UserPassword => AuthMode::UserPassword,
            AuthModeArg::SessionToken => AuthMode::SessionToken,
        }
    }
}
