use std::fmt;
use std::io;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum TruckError {
    InvalidHouseRange,
    InvalidShift,
    HourOutOfRange,
    DuplicateId,
    InvalidNumber(ParseIntError),
    Input(io::Error),
    Database(sqlx::Error),
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
            TruckError::Database(e) =>write!(f, "database error: {e}"),
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
    fn from( error: sqlx::Error)-> Self{
        TruckError::Database(error)
    }
}