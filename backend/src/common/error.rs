use std::fmt;

#[derive(Debug, PartialEq)]
pub enum ErrorType {
    NotFound,
    Internal,
    UniqueViolation,
}

#[derive(Debug)]
pub struct CustomError {
    pub err_type: ErrorType,
    pub message: String,
}

impl CustomError {
    pub fn new(message: &str, err_type: ErrorType) -> CustomError {
        CustomError { message: message.to_string(), err_type }
    }

    pub fn from_diesel_err(err: diesel::result::Error, context: &str) -> CustomError {
        CustomError::new(
            format!("{}: {}", context, err.to_string()).as_str(),
            match err {
                diesel::result::Error::DatabaseError(db_err, _) => {
                    match db_err {
                        diesel::result::DatabaseErrorKind::UniqueViolation => ErrorType::UniqueViolation,
                        _ => ErrorType::Internal,
                    }
                }
                diesel::result::Error::NotFound => ErrorType::NotFound,
                _ => {
                    ErrorType::Internal
                }
            },
        )
    }
}


impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}