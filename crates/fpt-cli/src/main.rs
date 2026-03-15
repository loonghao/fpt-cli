#![allow(clippy::result_large_err)]

mod cli;
mod output;
mod runner;
mod self_update;

use clap::Parser;
use cli::Cli;
use fpt_core::OutputFormat;
use std::process;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let output_format: OutputFormat = cli.output.into();
    let result = runner::run(cli).await;

    match result {
        Ok(value) => {
            output::print_stdout(&value, output_format);
            process::exit(0);
        }
        Err(error) => {
            output::print_stderr(&error.envelope(), output_format);
            process::exit(error.exit_code());
        }
    }
}
