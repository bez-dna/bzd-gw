use axum::{Router, extract::State, routing::post};
use bzd_messages_api::CreateMessageRequest;

use crate::app::{error::AppError, json::AppJson, state::AppState, user::AppUser};

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(create_message))
}

async fn create_message(
    State(AppState {
        messages_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
    AppJson(data): AppJson<create_message::Request>,
) -> Result<AppJson<create_message::Response>, AppError> {
    let mut req: CreateMessageRequest = data.into();
    req.user_id = Some(user.user_id.into());

    let res = messages_service_client
        .clone()
        .create_message(req)
        .await?
        .into_inner();

    Ok(AppJson(res.into()))
}

mod create_message {
    use bzd_messages_api::{
        CreateMessageRequest, CreateMessageResponse,
        create_message_request::{Regular, Starting, Tp},
    };
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct Request {
        pub text: String,
        pub code: String,
        pub message_id: Option<String>,
        pub topic_ids: Option<Vec<String>>,
    }

    impl From<Request> for CreateMessageRequest {
        fn from(req: Request) -> Self {
            Self {
                text: Some(req.text),
                user_id: None,
                code: Some(req.code),
                tp: if let Some(message_id) = req.message_id {
                    Some(Tp::Regular(Regular {
                        message_id: Some(message_id),
                    }))
                } else if let Some(topic_ids) = req.topic_ids {
                    Some(Tp::Starting(Starting { topic_ids }))
                } else {
                    None
                },
            }
        }
    }

    #[derive(Serialize)]
    pub struct Response {
        pub message: Message,
    }

    #[derive(Serialize)]
    pub struct Message {
        pub message_id: String,
    }

    impl From<CreateMessageResponse> for Response {
        fn from(res: CreateMessageResponse) -> Self {
            Self {
                message: Message {
                    message_id: res.message_id().into(),
                },
            }
        }
    }
}
