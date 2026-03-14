use clap::{Args, Parser, Subcommand, ValueEnum};
use fpt_core::OutputFormat;
use fpt_domain::{AuthMode, ConnectionOverrides};

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

#[derive(Debug, Parser)]
#[command(name = "fpt", version, about = "Flow Production Tracking CLI for OpenClaw")]
pub struct Cli {
    #[command(flatten)]
    pub connection: ConnectionArgs,

    #[arg(long, value_enum, global = true, default_value = "toon")]
    pub output: OutputFormatArg,


    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Args, Clone, Default)]
pub struct ConnectionArgs {
    #[arg(long, global = true)]
    pub site: Option<String>,

    #[arg(long = "auth-mode", value_enum, global = true)]
    pub auth_mode: Option<AuthModeArg>,

    #[arg(long = "script-name", global = true)]
    pub script_name: Option<String>,

    #[arg(long = "script-key", global = true)]
    pub script_key: Option<String>,

    #[arg(long, global = true)]
    pub username: Option<String>,

    #[arg(long, global = true)]
    pub password: Option<String>,

    #[arg(long = "auth-token", global = true)]
    pub auth_token: Option<String>,

    #[arg(long = "session-token", global = true)]
    pub session_token: Option<String>,

    #[arg(long = "api-version", global = true)]
    pub api_version: Option<String>,
}

impl From<ConnectionArgs> for ConnectionOverrides {
    fn from(value: ConnectionArgs) -> Self {
        Self {
            site: value.site,
            auth_mode: value.auth_mode.map(Into::into),
            script_name: value.script_name,
            script_key: value.script_key,
            username: value.username,
            password: value.password,
            auth_token: value.auth_token,
            session_token: value.session_token,
            api_version: value.api_version,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Capabilities,
    #[command(subcommand)]
    Inspect(InspectCommands),
    #[command(subcommand)]
    Auth(AuthCommands),
    #[command(subcommand)]
    Schema(SchemaCommands),
    #[command(subcommand)]
    Entity(EntityCommands),
}

#[derive(Debug, Subcommand)]
pub enum InspectCommands {
    Command { name: String },
}

#[derive(Debug, Subcommand)]
pub enum AuthCommands {
    Test,
}

#[derive(Debug, Subcommand)]
pub enum SchemaCommands {
    Entities,
    Fields { entity: String },
}

#[derive(Debug, Subcommand)]
pub enum EntityCommands {
    Get {
        entity: String,
        id: u64,
        #[arg(long, value_delimiter = ',')]
        fields: Vec<String>,
    },
    Find {
        entity: String,
        #[arg(long)]
        input: Option<String>,
        #[arg(long = "filter-dsl")]
        filter_dsl: Option<String>,
    },
    Create {
        entity: String,
        #[arg(long)]
        input: String,
        #[arg(long)]
        dry_run: bool,
    },
    Update {
        entity: String,
        id: u64,
        #[arg(long)]
        input: String,
        #[arg(long)]
        dry_run: bool,
    },
    Delete {
        entity: String,
        id: u64,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        yes: bool,
    },
    #[command(subcommand)]
    Batch(BatchEntityCommands),
}

#[derive(Debug, Subcommand)]
pub enum BatchEntityCommands {
    Get {
        entity: String,
        #[arg(long)]
        input: String,
    },
    Find {
        entity: String,
        #[arg(long)]
        input: String,
    },
    Create {
        entity: String,
        #[arg(long)]
        input: String,
        #[arg(long)]
        dry_run: bool,
    },
    Update {
        entity: String,
        #[arg(long)]
        input: String,
        #[arg(long)]
        dry_run: bool,
    },
    Delete {
        entity: String,
        #[arg(long)]
        input: String,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        yes: bool,
    },
}

