use axum::{Router, extract::State, routing::post};
use bzd_users_api::CreateSourceRequest;

use crate::app::{error::AppError, json::AppJson, state::AppState, user::AppUser};

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(create_source))
}

async fn create_source(
    State(AppState {
        sources_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
    AppJson(req): AppJson<create_source::Request>,
) -> Result<AppJson<create_source::Response>, AppError> {
    let res = sources_service_client
        .clone()
        .create_source(CreateSourceRequest {
            user_id: Some(user.user_id.into()),
            source_user_id: Some(req.user_id.into()),
        })
        .await?
        .into_inner();

    Ok(AppJson(res.into()))
}

mod create_source {
    use bzd_users_api::CreateSourceResponse;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct Request {
        pub user_id: String,
    }

    #[derive(Serialize)]
    pub struct Response {
        pub source_id: String,
    }

    impl From<CreateSourceResponse> for Response {
        fn from(res: CreateSourceResponse) -> Self {
            Self {
                source_id: res.source_id().into(),
            }
        }
    }
}
