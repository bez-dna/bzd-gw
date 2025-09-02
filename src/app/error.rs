use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::{debug, error};

#[derive(Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Status(#[from] tonic::Status),
    #[error(transparent)]
    Json(#[from] axum::extract::rejection::JsonRejection),
    #[error("NO_ENTITY")]
    NoEntity,
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
            AppError::NoEntity => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (code, String::from("")).into_response()
    }
}
