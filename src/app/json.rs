use axum::{
    Json,
    extract::FromRequest,
    response::{IntoResponse, Response},
};

use crate::app::error::AppError;

#[derive(FromRequest)]
#[from_request(via(Json), rejection(AppError))]
pub struct AppJson<T>(pub T);

impl<T> IntoResponse for AppJson<T>
where
    Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}
