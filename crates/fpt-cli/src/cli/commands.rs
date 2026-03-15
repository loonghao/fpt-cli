use clap::{Args, Parser, Subcommand};

use super::common::OutputFormatArg;
use super::connection::ConnectionArgs;

#[derive(Debug, Parser)]
#[command(
    name = "fpt",
    version,
    about = "Flow Production Tracking CLI for OpenClaw"
)]
pub struct Cli {
    #[command(flatten)]
    pub connection: ConnectionArgs,

    #[arg(long, value_enum, global = true, default_value = "json")]
    pub output: OutputFormatArg,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Capabilities,
    #[command(subcommand)]
    Inspect(InspectCommands),
    #[command(subcommand)]
    Auth(AuthCommands),
    #[command(subcommand)]
    Server(ServerCommands),
    #[command(subcommand)]
    Schema(SchemaCommands),
    #[command(subcommand)]
    Entity(EntityCommands),
    #[command(subcommand, name = "work-schedule")]
    WorkSchedule(WorkScheduleCommands),
    #[command(
        name = "self-update",
        about = "Check or install the released fpt binary for the current platform"
    )]
    SelfUpdate(SelfUpdateArgs),
}

#[derive(Debug, Args, Clone)]
pub struct SelfUpdateArgs {
    #[arg(long, help = "Only check whether a newer release is available")]
    pub check: bool,

    #[arg(
        long,
        help = "Install a specific release version instead of the latest release"
    )]
    pub version: Option<String>,

    #[arg(long, help = "Override the GitHub repository in owner/repo format")]
    pub repository: Option<String>,
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
pub enum ServerCommands {
    Info,
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
    #[command(name = "find-one")]
    FindOne {
        entity: String,
        #[arg(long)]
        input: Option<String>,
        #[arg(long = "filter-dsl")]
        filter_dsl: Option<String>,
    },
    Summarize {
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
    Revive {
        entity: String,
        id: u64,
        #[arg(long)]
        dry_run: bool,
    },
    #[command(subcommand)]
    Batch(BatchEntityCommands),
}

#[derive(Debug, Subcommand)]
pub enum WorkScheduleCommands {
    Read {
        #[arg(long)]
        input: String,
    },
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
