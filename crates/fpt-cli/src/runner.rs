use crate::cli::{
    AuthCommands, BatchEntityCommands, Cli, Commands, EntityCommands, InspectCommands,
    SchemaCommands, ServerCommands, WorkScheduleCommands,
};
use crate::self_update;
use fpt_core::{AppError, Result, read_json_input};
use fpt_domain::App;
use serde_json::Value;

pub async fn run(cli: Cli) -> Result<Value> {
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
        Commands::Server(command) => match command {
            ServerCommands::Info => app.server_info(connection).await,
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
            EntityCommands::FindOne {
                entity,
                input,
                filter_dsl,
            } => {
                let input = read_json_input(input.as_deref())?;
                app.entity_find_one(connection, &entity, input, filter_dsl)
                    .await
            }
            EntityCommands::Summarize { entity, input } => {
                let body = required_json_input(input)?;
                app.entity_summarize(connection, &entity, body).await
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
            EntityCommands::Revive {
                entity,
                id,
                dry_run,
            } => app.entity_revive(connection, &entity, id, dry_run).await,
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
                    app.entity_batch_create(connection, &entity, body, dry_run)
                        .await
                }
                BatchEntityCommands::Update {
                    entity,
                    input,
                    dry_run,
                } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_update(connection, &entity, body, dry_run)
                        .await
                }
                BatchEntityCommands::Delete {
                    entity,
                    input,
                    dry_run,
                    yes,
                } => {
                    let body = required_json_input(input)?;
                    app.entity_batch_delete(connection, &entity, body, dry_run, yes)
                        .await
                }
            },
        },
        Commands::WorkSchedule(command) => match command {
            WorkScheduleCommands::Read { input } => {
                let body = required_json_input(input)?;
                app.work_schedule_read(connection, body).await
            }
        },
        Commands::SelfUpdate(args) => self_update::run(args).await,
    }
}

fn required_json_input(input: String) -> Result<Value> {
    read_json_input(Some(&input))?.ok_or_else(|| AppError::invalid_input("missing JSON input"))
}
