use std::num::ParseIntError;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::typed_header::TypedHeaderRejection;
use thiserror::Error;
use tracing::{debug, error};

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error(transparent)]
    Json(#[from] axum::extract::rejection::JsonRejection),
    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("NO_ENTITY")]
    NoEntity,
    #[error("PARSE")]
    Parse,
    #[error("UNKNOWN")]
    Unknown,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        debug!("{}", self.to_string());

        let code = match self {
            AppError::Status(status) => match status.code() {
                tonic::Code::InvalidArgument => StatusCode::UNPROCESSABLE_ENTITY,
                tonic::Code::NotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::BAD_REQUEST,
            },
            AppError::Json(_) => StatusCode::BAD_REQUEST,
            AppError::NoEntity | AppError::Parse | AppError::Jwt(_) | AppError::Unknown => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        (code, String::from("")).into_response()
    }
}

impl From<ParseIntError> for AppError {
    fn from(_: ParseIntError) -> Self {
        Self::Unknown
    }
}

impl From<TypedHeaderRejection> for AppError {
    fn from(_: TypedHeaderRejection) -> Self {
        Self::Unknown
    }
}
