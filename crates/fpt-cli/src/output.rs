use fpt_core::{ErrorEnvelope, OutputFormat};
use serde::Serialize;
use serde_json::Value;

/// Format a serializable value according to the chosen output format.
fn format_value(value: &impl Serialize, output_format: OutputFormat) -> String {
    match output_format {
        OutputFormat::Toon => {
            toon_format::encode_default(value).unwrap_or_else(|_| {
                format!(
                    "{{\"serialization_error\":\"failed to encode as TOON\",\"fallback\":{}}}",
                    serde_json::to_string(value).unwrap_or_default()
                )
            })
        }
        OutputFormat::Json => {
            serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
        }
        OutputFormat::PrettyJson => {
            serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
        }
    }
}

pub fn print_stdout(value: &Value, output_format: OutputFormat) {
    println!("{}", format_value(value, output_format));
}

pub fn print_stderr(value: &ErrorEnvelope, output_format: OutputFormat) {
    eprintln!("{}", format_value(value, output_format));
}
