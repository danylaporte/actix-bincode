use actix_web::{error::PayloadError, http::StatusCode, BaseHttpResponse, ResponseError};
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct BincodeError(bincode::Error);

#[derive(Debug)]
pub enum BincodePayloadError {
    /// Payload size is bigger than allowed. (default: 32kB)
    Overflow,
    /// Content type error
    ContentType,
    /// Deserialize error
    Deserialize(bincode::Error),
    /// Payload error
    Payload(PayloadError),
}

impl From<bincode::Error> for BincodePayloadError {
    fn from(e: bincode::Error) -> Self {
        Self::Deserialize(e)
    }
}

impl From<PayloadError> for BincodePayloadError {
    fn from(e: PayloadError) -> Self {
        Self::Payload(e)
    }
}

impl fmt::Display for BincodePayloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Overflow => writeln!(f, "Bincode payload size is bigger than allowed"),
            Self::ContentType => writeln!(f, "Content type error"),
            Self::Deserialize(inner) => {
                writeln!(f, "Bincode deserialize error: {}", inner)
            }
            Self::Payload(inner) => {
                writeln!(f, "Error that occur during reading payload: {:?}", inner)
            }
        }
    }
}

impl Error for BincodePayloadError {}

/// Return `BadRequest` for `BincodePayloadError`
impl ResponseError for BincodePayloadError {
    fn error_response(&self) -> BaseHttpResponse<actix_web::dev::Body> {
        match *self {
            Self::Overflow => BaseHttpResponse::new(StatusCode::PAYLOAD_TOO_LARGE),
            _ => BaseHttpResponse::new(StatusCode::BAD_REQUEST),
        }
    }
}

impl fmt::Display for BincodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for BincodeError {}

impl ResponseError for BincodeError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<bincode::Error> for BincodeError {
    fn from(e: bincode::Error) -> Self {
        Self(e)
    }
}
