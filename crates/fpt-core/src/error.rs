use serde::Serialize;
use serde_json::Value;
use std::fmt::{Display, Formatter};

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidInput,
    AuthFailed,
    NetworkError,
    ApiError,
    PolicyBlocked,
    UnsupportedCapability,
    NotImplemented,
    InternalError,
}

impl ErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidInput => "INVALID_INPUT",
            Self::AuthFailed => "AUTH_FAILED",
            Self::NetworkError => "NETWORK_ERROR",
            Self::ApiError => "API_ERROR",
            Self::PolicyBlocked => "POLICY_BLOCKED",
            Self::UnsupportedCapability => "UNSUPPORTED_CAPABILITY",
            Self::NotImplemented => "NOT_IMPLEMENTED",
            Self::InternalError => "INTERNAL_ERROR",
        }
    }

    pub const fn exit_code(self) -> i32 {
        match self {
            Self::InvalidInput => 10,
            Self::AuthFailed => 20,
            Self::NetworkError => 30,
            Self::ApiError => 40,
            Self::PolicyBlocked => 50,
            Self::UnsupportedCapability => 60,
            Self::NotImplemented => 61,
            Self::InternalError => 70,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorEnvelope {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AppError {
    code: ErrorCode,
    message: String,
    details: Option<Value>,
    retryable: bool,
    transport: Option<String>,
}

impl AppError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
            retryable: false,
            transport: None,
        }
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidInput, message)
    }

    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::AuthFailed, message)
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NetworkError, message)
    }

    pub fn api(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::ApiError, message)
    }

    pub fn policy_blocked(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::PolicyBlocked, message)
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::UnsupportedCapability, message)
    }

    pub fn not_implemented(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotImplemented, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::InternalError, message)
    }

    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    pub fn with_transport(mut self, transport: impl Into<String>) -> Self {
        self.transport = Some(transport.into());
        self
    }

    pub fn retryable(mut self, retryable: bool) -> Self {
        self.retryable = retryable;
        self
    }

    pub fn envelope(&self) -> ErrorEnvelope {
        ErrorEnvelope {
            code: self.code.as_str(),
            message: self.message.clone(),
            details: self.details.clone(),
            retryable: self.retryable,
            transport: self.transport.clone(),
        }
    }

    pub const fn exit_code(&self) -> i32 {
        self.code.exit_code()
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AppError {}
