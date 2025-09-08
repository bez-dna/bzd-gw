pub mod settings;

use axum::{
    Router,
    extract::State,
    routing::{get, post},
};
use bzd_users_api::GetUserRequest;

use crate::app::{error::AppError, json::AppJson, state::AppState, user::AppUser};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/join", post(join))
        .route("/complete", post(complete))
        .route("/me", get(me))
}

async fn join(
    State(AppState {
        auth_service_client,
        ..
    }): State<AppState>,
    AppJson(data): AppJson<join::Request>,
) -> Result<AppJson<join::Response>, AppError> {
    let request: bzd_users_api::JoinRequest = data.try_into()?;

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
        pub phone_number: String,
    }

    impl TryFrom<Request> for bzd_users_api::JoinRequest {
        type Error = AppError;

        fn try_from(req: Request) -> Result<Self, Self::Error> {
            Ok(Self {
                phone_number: Some(req.phone_number.parse::<i64>()?),
            })
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

async fn me(
    State(AppState {
        users_service_client,
        ..
    }): State<AppState>,
    user: Option<AppUser>,
) -> Result<AppJson<me::Response>, AppError> {
    let res = match user {
        Some(user) => {
            let req = GetUserRequest {
                user_id: Some(user.user_id),
            };

            users_service_client
                .clone()
                .get_user(req)
                .await?
                .into_inner()
                .try_into()?
        }
        None => me::Response { user: None },
    };

    Ok(AppJson(res))
}

mod me {
    use bzd_users_api::GetUserResponse;
    use serde::Serialize;

    use crate::app::error::AppError;

    #[derive(Serialize)]
    pub struct Response {
        pub user: Option<User>,
    }

    #[derive(Serialize)]
    pub struct User {
        pub user_id: String,
    }

    impl TryFrom<GetUserResponse> for Response {
        type Error = AppError;

        fn try_from(res: GetUserResponse) -> Result<Self, Self::Error> {
            let user = res.user.ok_or(AppError::NoEntity)?;

            Ok(Self {
                user: Some(User {
                    user_id: user.user_id().into(),
                }),
            })
        }
    }
}
