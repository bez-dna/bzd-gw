use axum::{
    Router,
    extract::State,
    routing::{get, post},
};
use bzd_users_api::users::{GetUserRequest, GetUserResponse};
use serde_json::Value;

use crate::app::{current_user::CurrentUser, error::AppError, json::AppJson, state::AppState};

pub mod settings;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/join", post(join))
        .route("/login", post(login))
        .route("/complete", post(complete))
        .route("/me", get(me))
}

async fn join(
    State(AppState {
        auth_service_client,
        ..
    }): State<AppState>,
    AppJson(data): AppJson<join::Request>,
) -> Result<AppJson<Value>, AppError> {
    let request: bzd_users_api::auth::JoinRequest = data.try_into()?;

    let res = auth_service_client
        .clone()
        .join(request)
        .await?
        .into_inner();

    Ok(AppJson(serde_json::from_str(res.response())?))
}

mod join {
    use serde::Deserialize;

    use crate::app::error::AppError;

    #[derive(Deserialize)]
    pub struct Request {
        pub login: String,
    }

    impl TryFrom<Request> for bzd_users_api::auth::JoinRequest {
        type Error = AppError;

        fn try_from(req: Request) -> Result<Self, Self::Error> {
            Ok(Self {
                login: Some(req.login),
            })
        }
    }
}

async fn login(
    State(AppState {
        auth_service_client,
        ..
    }): State<AppState>,
    AppJson(data): AppJson<login::Request>,
) -> Result<AppJson<login::Response>, AppError> {
    dbg!(&data);

    let request: bzd_users_api::auth::LoginRequest = data.into();

    let response = auth_service_client
        .clone()
        .login(request)
        .await?
        .into_inner();

    Ok(AppJson(response.into()))
}

mod login {
    use bzd_users_api::auth::{LoginRequest, LoginResponse};
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};

    #[derive(Deserialize, Serialize, Debug)]
    pub struct Request {
        pub credential_id: Value,
        pub client_data: Value,
        pub signature: Value,
        pub authenticator_data: Value,
    }

    impl From<Request> for LoginRequest {
        fn from(req: Request) -> Self {
            Self {
                request: Some(json!(req).to_string()),
            }
        }
    }

    #[derive(Serialize)]
    pub struct Response {
        pub jwt: String,
    }

    impl From<LoginResponse> for Response {
        fn from(res: LoginResponse) -> Self {
            Self {
                jwt: res.jwt().into(),
            }
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
    dbg!(&data);

    let request: bzd_users_api::auth::CompleteRequest = data.into();

    let response = auth_service_client
        .clone()
        .complete(request)
        .await?
        .into_inner();

    Ok(AppJson(response.into()))
}

mod complete {
    use bzd_users_api::auth::{CompleteRequest, CompleteResponse};
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};

    #[derive(Deserialize, Serialize, Debug)]
    pub struct Request {
        pub credential_id: Value,
        pub client_data: Value,
        pub attestation_object: Value,
    }

    impl From<Request> for CompleteRequest {
        fn from(req: Request) -> Self {
            Self {
                request: Some(json!(req).to_string()),
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
    user: CurrentUser,
) -> Result<AppJson<me::Response>, AppError> {
    // Тут неправильно использовать GetUser, потому что теперь GetXxx это просто каноничный способ получить данные
    // о сущности, а GetMe должен получить свой метод получения инфы от текущем юзере
    // ну и конечно после GetMe не будет вызова GetUser (типа если уж быть совсем каноничным задротом),
    // но я тут как-бы-типа-того дух стартапа держу ващет
    // и поэтому наличие user_id гарантирует наличие корректного id текущего юзера,
    // поэтом у сразу в GetUser (нужно переделать)

    let res = match user.user_id {
        Some(user_id) => users_service_client
            .clone()
            .get_user(GetUserRequest {
                user_id: Some(user_id),
            })
            .await?
            .into_inner(),
        None => GetUserResponse::default(),
    };

    Ok(AppJson(res.try_into()?))
}

mod me {
    use bzd_users_api::users::GetUserResponse;
    use serde::Serialize;

    use crate::app::error::AppError;

    #[derive(Serialize)]
    pub struct Response {
        pub user: Option<User>,
    }

    #[derive(Serialize)]
    pub struct User {
        pub user_id: String,
        pub name: String,
        pub abbr: String,
        pub color: String,
    }

    impl TryFrom<GetUserResponse> for Response {
        type Error = AppError;

        fn try_from(res: GetUserResponse) -> Result<Self, Self::Error> {
            Ok(Self {
                user: res.user.map(|user| User {
                    user_id: user.user_id().into(),
                    name: user.name().into(),
                    abbr: user.abbr().into(),
                    color: user.color().into(),
                }),
            })
        }
    }
}
