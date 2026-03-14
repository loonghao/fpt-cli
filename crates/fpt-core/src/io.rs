use crate::error::{AppError, Result};
use serde_json::Value;
use std::fs;
use std::io::Read;

pub fn read_json_input(input: Option<&str>) -> Result<Option<Value>> {
    match input {
        None => Ok(None),
        Some(source) => read_from_source(source).map(Some),
    }
}

fn read_from_source(source: &str) -> Result<Value> {
    if source == "@-" {
        let mut buffer = String::new();
        std::io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|error| AppError::invalid_input(format!("failed to read stdin: {error}")))?;

        return parse_json(&buffer, "stdin");
    }

    if let Some(path) = source.strip_prefix('@') {
        let content = fs::read_to_string(path)
            .map_err(|error| AppError::invalid_input(format!("failed to read input file `{path}`: {error}")))?;

        return parse_json(&content, path);
    }

    parse_json(source, "inline JSON")
}

fn parse_json(raw: &str, label: &str) -> Result<Value> {
    serde_json::from_str(raw)
        .map_err(|error| AppError::invalid_input(format!("failed to parse {label}: {error}")))
}

