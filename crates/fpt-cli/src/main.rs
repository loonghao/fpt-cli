mod cli;

use clap::Parser;
use cli::{
    AuthCommands, BatchEntityCommands, Cli, Commands, EntityCommands, InspectCommands,
    SchemaCommands,
};


use fpt_core::{read_json_input, AppError, ErrorEnvelope, OutputFormat, Result};
use fpt_domain::App;
use serde_json::Value;
use std::process;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let output_format: OutputFormat = cli.output.into();
    let result = run(cli).await;

    match result {
        Ok(value) => {
            print_stdout(&value, output_format);
            process::exit(0);
        }
        Err(error) => {
            print_stderr(&error.envelope(), output_format);
            process::exit(error.exit_code());
        }
    }
}

async fn run(cli: Cli) -> Result<Value> {
    let app = App::default();
    let connection = cli.connection.clone().into();

    match cli.command {
        Commands::Capabilities => Ok(app.capabilities()),
        Commands::Inspect(command) => match command {
            InspectCommands::Command { name } => app.inspect_command(&name),
        },
        Commands::Auth(command) => match command {
            AuthCommands::Test => app.auth_test(connection).await,
        },
        Commands::Schema(command) => match command {
            SchemaCommands::Entities => app.schema_entities(connection).await,
            SchemaCommands::Fields { entity } => app.schema_fields(connection, &entity).await,
        },
        Commands::Entity(command) => match command {
            EntityCommands::Get { entity, id, fields } => {
                let fields = (!fields.is_empty()).then_some(fields);
                app.entity_get(connection, &entity, id, fields).await
            }
            EntityCommands::Find {
                entity,
                input,
                filter_dsl,
            } => {
                let input = read_json_input(input.as_deref())?;
                app.entity_find(connection, &entity, input, filter_dsl).await
            }
            EntityCommands::Create {
                entity,
                input,
                dry_run,
            } => {
                let body = required_json_input(input)?;
                app.entity_create(connection, &entity, body, dry_run).await
            }
            EntityCommands::Update {
                entity,
                id,
                input,
                dry_run,
            } => {
                let body = required_json_input(input)?;
                app.entity_update(connection, &entity, id, body, dry_run).await
            }
            EntityCommands::Delete {
                entity,
                id,
                dry_run,
                yes,
            } => app.entity_delete(connection, &entity, id, dry_run, yes).await,
            EntityCommands::Batch(command) => match command {
                BatchEntityCommands::Get { entity, input } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_get(connection, &entity, body).await
                }
                BatchEntityCommands::Find { entity, input } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_find(connection, &entity, body).await
                }
                BatchEntityCommands::Create {
                    entity,
                    input,
                    dry_run,
                } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_create(connection, &entity, body, dry_run).await
                }
                BatchEntityCommands::Update {
                    entity,
                    input,
                    dry_run,
                } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_update(connection, &entity, body, dry_run).await
                }
                BatchEntityCommands::Delete {
                    entity,
                    input,
                    dry_run,
                    yes,
                } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_delete(connection, &entity, body, dry_run, yes).await
                }
            },
        },

    }
}

fn required_json_input(input: String) -> Result<Value> {
    read_json_input(Some(&input))?
        .ok_or_else(|| AppError::invalid_input("缺少 JSON 输入"))
}

fn print_stdout(value: &Value, output_format: OutputFormat) {
    match output_format {
        OutputFormat::Toon => println!("{}", toon_format::encode_default(value).expect("serialize stdout as TOON")),
        OutputFormat::Json => println!("{}", serde_json::to_string(value).expect("serialize stdout")),
        OutputFormat::PrettyJson => {
            println!("{}", serde_json::to_string_pretty(value).expect("serialize stdout"))
        }
    }
}

fn print_stderr(value: &ErrorEnvelope, output_format: OutputFormat) {
    match output_format {
        OutputFormat::Toon => eprintln!("{}", toon_format::encode_default(value).expect("serialize stderr as TOON")),
        OutputFormat::Json => eprintln!("{}", serde_json::to_string(value).expect("serialize stderr")),
        OutputFormat::PrettyJson => {
            eprintln!("{}", serde_json::to_string_pretty(value).expect("serialize stderr"))
        }
    }
}



