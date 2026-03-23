use fpt_core::{ErrorEnvelope, OutputFormat};
use serde_json::Value;

pub fn print_stdout(value: &Value, output_format: OutputFormat) {
    match output_format {
        OutputFormat::Toon => println!(
            "{}",
            toon_format::encode_default(value)
                .unwrap_or_else(|_| format!("{{\"serialization_error\":\"failed to encode as TOON\",\"fallback\":{}}}", serde_json::to_string(value).unwrap_or_default()))
        ),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
        ),
        OutputFormat::PrettyJson => {
            println!(
                "{}",
                serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
            )
        }
    }
}

pub fn print_stderr(value: &ErrorEnvelope, output_format: OutputFormat) {
    match output_format {
        OutputFormat::Toon => eprintln!(
            "{}",
            toon_format::encode_default(value)
                .unwrap_or_else(|_| format!("{{\"serialization_error\":\"failed to encode as TOON\",\"fallback\":{}}}", serde_json::to_string(value).unwrap_or_default()))
        ),
        OutputFormat::Json => eprintln!(
            "{}",
            serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string())
        ),
        OutputFormat::PrettyJson => {
            eprintln!(
                "{}",
                serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string())
            )
        }
    }
}
