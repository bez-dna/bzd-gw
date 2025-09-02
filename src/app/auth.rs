use axum::{Router, extract::State, routing::post};

use crate::app::{error::AppError, json::AppJson, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/join", post(join))
        .route("/complete", post(complete))
}

async fn join(
    State(AppState {
        auth_service_client,
        ..
    }): State<AppState>,
    AppJson(data): AppJson<join::Request>,
) -> Result<AppJson<join::Response>, AppError> {
    let request: bzd_users_api::JoinRequest = data.into();

    let response = auth_service_client
        .clone()
        .join(request)
        .await?
        .into_inner();

    Ok(AppJson(response.try_into()?))
}

mod join {
    use bzd_users_api::JoinResponse;
    use serde::{Deserialize, Serialize};

    use crate::app::error::AppError;

    #[derive(Deserialize)]
    pub struct Request {
        pub phone_number: i64,
    }

    impl From<Request> for bzd_users_api::JoinRequest {
        fn from(req: Request) -> Self {
            Self {
                phone_number: Some(req.phone_number),
            }
        }
    }

    #[derive(Serialize)]
    pub struct Response {
        pub verification: Verification,
    }

    #[derive(Serialize)]
    pub struct Verification {
        pub verification_id: String,
    }

    impl TryFrom<JoinResponse> for Response {
        type Error = AppError;

        fn try_from(res: JoinResponse) -> Result<Self, Self::Error> {
            Ok(Self {
                verification: Verification {
                    verification_id: res
                        .verification
                        .ok_or(AppError::NoEntity)?
                        .verification_id()
                        .into(),
                },
            })
        }
    }
}

async fn complete(
    State(AppState {
        auth_service_client,
        ..
    }): State<AppState>,
    AppJson(data): AppJson<complete::Request>,
) -> Result<AppJson<complete::Response>, AppError> {
    let request: bzd_users_api::CompleteRequest = data.into();

    let response = auth_service_client
        .clone()
        .complete(request)
        .await?
        .into_inner();

    Ok(AppJson(response.into()))
}

mod complete {
    use bzd_users_api::{CompleteRequest, CompleteResponse};
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct Request {
        pub verification_id: String,
        pub code: String,
    }

    impl From<Request> for CompleteRequest {
        fn from(req: Request) -> Self {
            Self {
                verification_id: Some(req.verification_id),
                code: Some(req.code),
            }
        }
    }

    #[derive(Serialize)]
    pub struct Response {
        pub jwt: String,
    }

    impl From<CompleteResponse> for Response {
        fn from(res: CompleteResponse) -> Self {
            Self {
                jwt: res.jwt().into(),
            }
        }
    }
}
