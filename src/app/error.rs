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
    #[error("COMMON")]
    Common,
    #[error("INTERNAL")]
    Internal,
}

/*
Логика такая: если Error на процессинге внешних данных, то BAD_REQUEST,
а если на процессинге внутренних данных — INTERNAL_SERVER_ERROR
 */

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        debug!("{}", self.to_string());

        let code = match self {
            AppError::Status(status) => match status.code() {
                tonic::Code::InvalidArgument => StatusCode::UNPROCESSABLE_ENTITY,
                tonic::Code::NotFound => StatusCode::NOT_FOUND,
                _ => StatusCode::BAD_REQUEST,
            },
            AppError::Common | AppError::Jwt(_) | AppError::Json(_) => StatusCode::BAD_REQUEST,
            AppError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (code, String::from("")).into_response()
    }
}

impl From<ParseIntError> for AppError {
    fn from(_: ParseIntError) -> Self {
        Self::Common
    }
}

impl From<TypedHeaderRejection> for AppError {
    fn from(_: TypedHeaderRejection) -> Self {
        Self::Common
    }
}
