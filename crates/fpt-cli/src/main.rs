#![allow(clippy::result_large_err)]

mod cli;
mod config;
mod output;
mod runner;
mod self_update;

use clap::Parser;
use cli::Cli;
use fpt_core::OutputFormat;
use serde_json::Value;
use std::process;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let output_format: OutputFormat = cli.output.into();
    let result = runner::run(cli).await;

    match result {
        Ok(value) => {
            output::print_stdout(&value, output_format);
            let exit_code = logical_failure_exit_code(&value);
            process::exit(exit_code);
        }
        Err(error) => {
            output::print_stderr(&error.envelope(), output_format);
            process::exit(error.exit_code());
        }
    }
}

/// Returns a non-zero exit code when the response signals a logical failure:
///
/// - `{"exception": true, ...}` — RPC-level exception (e.g. entity summarize)
/// - `{"ok": false, ...}` — top-level operation failure
/// - `{"ok": false, "failure_count": N, ...}` where N > 0 — batch operation with failures
///
/// Returns `0` when the response indicates success or is not a recognized failure shape.
fn logical_failure_exit_code(value: &Value) -> i32 {
    let Some(obj) = value.as_object() else {
        return 0;
    };

    // RPC exception: {"exception": true, ...}
    if obj.get("exception").and_then(Value::as_bool) == Some(true) {
        return 1;
    }

    // Top-level ok: false
    if obj.get("ok").and_then(Value::as_bool) == Some(false) {
        return 1;
    }

    0
}
