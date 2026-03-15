use fpt_core::{ErrorEnvelope, OutputFormat};
use serde_json::Value;

pub fn print_stdout(value: &Value, output_format: OutputFormat) {
    match output_format {
        OutputFormat::Toon => println!(
            "{}",
            toon_format::encode_default(value).expect("serialize stdout as TOON")
        ),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string(value).expect("serialize stdout")
        ),
        OutputFormat::PrettyJson => {
            println!(
                "{}",
                serde_json::to_string_pretty(value).expect("serialize stdout")
            )
        }
    }
}

pub fn print_stderr(value: &ErrorEnvelope, output_format: OutputFormat) {
    match output_format {
        OutputFormat::Toon => eprintln!(
            "{}",
            toon_format::encode_default(value).expect("serialize stderr as TOON")
        ),
        OutputFormat::Json => eprintln!(
            "{}",
            serde_json::to_string(value).expect("serialize stderr")
        ),
        OutputFormat::PrettyJson => {
            eprintln!(
                "{}",
                serde_json::to_string_pretty(value).expect("serialize stderr")
            )
        }
    }
}
