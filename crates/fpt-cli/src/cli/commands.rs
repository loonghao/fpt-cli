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
    #[command(subcommand)]
    Upload(UploadCommands),
    #[command(subcommand)]
    Download(DownloadCommands),
    #[command(subcommand)]
    Thumbnail(ThumbnailCommands),
    #[command(subcommand)]
    Activity(ActivityCommands),
    #[command(subcommand, name = "event-log")]
    EventLog(EventLogCommands),
    #[command(subcommand)]
    Preferences(PreferencesCommands),
    #[command(subcommand)]
    Followers(FollowersCommands),
    #[command(subcommand)]
    Note(NoteCommands),
    #[command(subcommand)]
    Hierarchy(HierarchyCommands),
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
    #[command(name = "field-create", about = "Create a new custom field on an entity type")]
    FieldCreate {
        entity: String,
        #[arg(long)]
        input: String,
    },
    #[command(name = "field-update", about = "Update properties of an existing custom field")]
    FieldUpdate {
        entity: String,
        field_name: String,
        #[arg(long)]
        input: String,
    },
    #[command(name = "field-delete", about = "Delete a custom field from an entity type")]
    FieldDelete {
        entity: String,
        field_name: String,
    },
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

#[derive(Debug, Subcommand)]
pub enum UploadCommands {
    #[command(name = "url", about = "Get a pre-signed upload URL for an entity field attachment")]
    Url {
        entity: String,
        id: u64,
        field_name: String,
        file_name: String,
        #[arg(long, help = "MIME content type of the file being uploaded")]
        content_type: Option<String>,
        #[arg(long, help = "Request a multipart upload URL for large files")]
        multipart: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum DownloadCommands {
    #[command(name = "url", about = "Get a pre-signed download URL for an entity field attachment")]
    Url {
        entity: String,
        id: u64,
        field_name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum ThumbnailCommands {
    #[command(name = "url", about = "Get the thumbnail image URL for an entity record")]
    Url {
        entity: String,
        id: u64,
    },
}

#[derive(Debug, Subcommand)]
pub enum ActivityCommands {
    #[command(name = "stream", about = "Fetch the activity stream for a specific entity record")]
    Stream {
        entity: String,
        id: u64,
        #[arg(long, help = "Optional query parameters as JSON (page, fields, entity_fields, etc.)")]
        input: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum EventLogCommands {
    #[command(name = "entries", about = "Query ShotGrid event log entries")]
    Entries {
        #[arg(long, help = "Optional query parameters as JSON (fields, sort, page, etc.)")]
        input: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum PreferencesCommands {
    #[command(name = "get", about = "Read site-level ShotGrid preferences")]
    Get,
}

#[derive(Debug, Subcommand)]
pub enum FollowersCommands {
    #[command(name = "list", about = "List all followers of an entity record")]
    List {
        entity: String,
        id: u64,
    },
    #[command(name = "follow", about = "Add a user as a follower of an entity record")]
    Follow {
        entity: String,
        id: u64,
        #[arg(long, help = "User JSON object with type and id, e.g. '{\"type\":\"HumanUser\",\"id\":456}'")]
        input: String,
    },
    #[command(name = "unfollow", about = "Remove a user from the followers of an entity record")]
    Unfollow {
        entity: String,
        id: u64,
        #[arg(long, help = "User JSON object with type and id")]
        input: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum NoteCommands {
    #[command(name = "threads", about = "Get the reply thread contents for a Note record")]
    Threads {
        note_id: u64,
        #[arg(long, help = "Optional query parameters as JSON (fields, entity_fields, etc.)")]
        input: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum HierarchyCommands {
    #[command(name = "search", about = "Search the ShotGrid entity hierarchy navigation tree")]
    Search {
        #[arg(long, help = "Hierarchy search body as JSON")]
        input: String,
    },
}
