use std::fmt;
use std::io;
use std::num::ParseIntError;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
pub enum TruckError {
    InvalidHouseRange,
    InvalidShift,
    HourOutOfRange,
    DuplicateId,
    InvalidNumber(ParseIntError),
    Input(io::Error),
    Database(sqlx::Error),
    NotFound,
    Unauthorized,
    Forbidden,
    Internal(String),
    InvalidInput(String),
    Conflict(String),
}

impl fmt::Display for TruckError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TruckError::InvalidHouseRange => write!(f, "first house must not exceed last house"),
            TruckError::InvalidShift => write!(f, "shift start and end cannot be the same"),
            TruckError::HourOutOfRange => write!(f, "hours must be between 0 and 23"),
            TruckError::DuplicateId => write!(f, "a truck with that id already exists"),
            TruckError::InvalidNumber(e) => write!(f, "invalid number: {e}"),
            TruckError::Input(e) => write!(f, "input error: {e}"),
            TruckError::Database(e) => write!(f, "database error: {e}"),
            TruckError::NotFound => write!(f, "no truck covers that address"),
            TruckError::Unauthorized => write!(f, "invalid credentials or token"),
            TruckError::Forbidden => write!(f, "you do not have permission to do that"),
            TruckError::Internal(e) => write!(f, "internal error: {e}"),
            TruckError::InvalidInput(m) => write!(f, "{m}"),
            TruckError::Conflict(m) => write!(f, "{m}"),
        }
    }
}

impl From<ParseIntError> for TruckError {
    fn from(error: ParseIntError) -> Self {
        TruckError::InvalidNumber(error)
    }
}

impl From<io::Error> for TruckError {
    fn from(error: io::Error) -> Self {
        TruckError::Input(error)
    }
}

impl From<sqlx::Error> for TruckError {
    fn from(error: sqlx::Error) -> Self {
        TruckError::Database(error)
    }
}

impl IntoResponse for TruckError {
    fn into_response(self) -> Response {
        match &self {
            TruckError::NotFound => (StatusCode::NOT_FOUND, self.to_string()).into_response(),
            TruckError::InvalidNumber(_)
            | TruckError::InvalidHouseRange
            | TruckError::InvalidShift
            | TruckError::HourOutOfRange => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
            TruckError::DuplicateId => (StatusCode::CONFLICT, self.to_string()).into_response(),
            // Never leak database or I/O details to clients: log them, send a generic message.
            TruckError::Database(_) | TruckError::Input(_) | TruckError::Internal(_) => {
                tracing::error!("internal error: {self}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
                    .into_response()
            }
            TruckError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
            }
            TruckError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()).into_response(),
            TruckError::InvalidInput(_) => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
            TruckError::Conflict(_) => (StatusCode::CONFLICT, self.to_string()).into_response(),
        }
    }
}
