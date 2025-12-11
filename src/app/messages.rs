use std::collections::HashSet;

use axum::{
    Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use bzd_messages_api::{
    CreateMessageRequest, GetMessageMessagesRequest, GetMessageRequest, GetMessagesRequest,
    GetUserMessagesRequest,
};
use bzd_users_api::GetUsersRequest;

use crate::app::{current_user::CurrentUser, error::AppError, json::AppJson, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_message))
        .route("/", get(get_user_messages))
        .route("/{message_id}", get(get_message))
        .route("/{message_id}/messages", get(get_message_messages))
}

async fn create_message(
    State(AppState {
        messages_service_client,
        ..
    }): State<AppState>,
    user: CurrentUser,
    AppJson(data): AppJson<create_message::Request>,
) -> Result<AppJson<create_message::Response>, AppError> {
    let mut req: CreateMessageRequest = data.into();
    req.current_user_id = user.user_id;

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
                current_user_id: None,
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

async fn get_user_messages(
    State(AppState {
        messages_service_client,
        users_service_client,
        ..
    }): State<AppState>,
    user: CurrentUser,
    Query(req): Query<get_user_messages::Request>,
) -> Result<AppJson<get_user_messages::Response>, AppError> {
    let get_user_messages = messages_service_client
        .clone()
        .get_user_messages(GetUserMessagesRequest {
            user_id: user.user_id.into(),
            cursor_message_id: req.cursor_message_id.into(),
        })
        .await?
        .into_inner();

    let get_messages = messages_service_client
        .clone()
        .get_messages(GetMessagesRequest {
            message_ids: get_user_messages.message_ids.clone(),
        })
        .await?
        .into_inner();

    let user_ids: HashSet<String> = get_messages
        .messages
        .iter()
        .map(|it| it.user_id().into())
        .collect();

    let get_users = users_service_client
        .clone()
        .get_users(GetUsersRequest {
            user_ids: user_ids.clone().into_iter().collect(),
        })
        .await?
        .into_inner();

    Ok(AppJson(
        (get_user_messages, get_messages, get_users).try_into()?,
    ))
}

mod get_user_messages {
    use std::collections::HashMap;

    use bzd_messages_api::{GetMessagesResponse, GetUserMessagesResponse, get_messages_response};
    use bzd_users_api::{GetUsersResponse, get_users_response};
    use serde::{Deserialize, Serialize};

    use crate::app::error::AppError;

    #[derive(Deserialize)]
    pub struct Request {
        pub cursor_message_id: Option<String>,
    }

    #[derive(Serialize)]
    pub struct Response {
        pub messages: Vec<Message>,
        pub cursor_message_id: Option<String>,
    }

    #[derive(Serialize)]
    pub struct Message {
        pub message_id: String,
        pub text: String,
        pub user: User,
    }

    #[derive(Serialize)]
    pub struct User {
        pub user_id: String,
        pub name: String,
        pub abbr: String,
        pub color: String,
    }

    type Responses = (
        GetUserMessagesResponse,
        GetMessagesResponse,
        GetUsersResponse,
    );

    type Users = HashMap<String, get_users_response::User>;
    type Messages = HashMap<String, get_messages_response::Message>;

    impl TryFrom<Responses> for Response {
        type Error = AppError;

        fn try_from(
            (get_user_messages, get_messages, get_users): Responses,
        ) -> Result<Self, Self::Error> {
            let users: Users = get_users
                .users
                .into_iter()
                .map(|it| (it.user_id().into(), it))
                .collect();

            let messages: Messages = get_messages
                .messages
                .into_iter()
                .map(|it| (it.message_id().into(), it))
                .collect();

            Ok(Self {
                messages: get_user_messages
                    .message_ids
                    .into_iter()
                    .map(|message_id| (message_id, &messages, &users).try_into())
                    .collect::<Result<_, _>>()?,
                cursor_message_id: get_user_messages.cursor_message_id,
            })
        }
    }

    impl TryFrom<(String, &Messages, &Users)> for Message {
        type Error = AppError;

        fn try_from(
            (message_id, messages, users): (String, &Messages, &Users),
        ) -> Result<Self, Self::Error> {
            let message = messages
                .get(&message_id)
                .ok_or(AppError::Unreachable)?
                .to_owned();

            let user = users
                .get(&message.user_id().to_string())
                .ok_or(AppError::Unreachable)?
                .to_owned();

            Ok(Self {
                message_id: message.message_id().into(),
                text: message.text().into(),
                user: User {
                    user_id: user.user_id().into(),
                    name: user.name().into(),
                    abbr: user.abbr().into(),
                    color: user.color().into(),
                },
            })
        }
    }
}

async fn get_message(
    Path(message_id): Path<String>,
    State(AppState {
        messages_service_client,
        ..
    }): State<AppState>,
) -> Result<AppJson<get_message::Response>, AppError> {
    let get_message = messages_service_client
        .clone()
        .get_message(GetMessageRequest {
            message_id: message_id.into(),
        })
        .await?
        .into_inner();

    Ok(AppJson(get_message.try_into()?))
}

mod get_message {
    use bzd_messages_api::GetMessageResponse;
    use serde::Serialize;

    use crate::app::error::AppError;

    #[derive(Serialize)]
    pub struct Response {
        message: Message,
    }

    #[derive(Serialize)]
    struct Message {
        message_id: String,
        text: String,
    }

    impl TryFrom<GetMessageResponse> for Response {
        type Error = AppError;

        fn try_from(get_message: GetMessageResponse) -> Result<Self, Self::Error> {
            let message = get_message.message.ok_or(AppError::Unreachable)?;

            Ok(Self {
                message: Message {
                    message_id: message.message_id().into(),
                    text: message.text().into(),
                },
            })
        }
    }
}

async fn get_message_messages(
    Path(message_id): Path<String>,
    State(AppState {
        messages_service_client,
        users_service_client,
        ..
    }): State<AppState>,
    Query(req): Query<get_message_messages::Request>,
) -> Result<AppJson<get_message_messages::Response>, AppError> {
    let get_message_messages = messages_service_client
        .clone()
        .get_message_messages(GetMessageMessagesRequest {
            message_id: message_id.into(),
            cursor_message_id: req.cursor_message_id.into(),
        })
        .await?
        .into_inner();

    let get_messages = messages_service_client
        .clone()
        .get_messages(GetMessagesRequest {
            message_ids: get_message_messages.message_ids.clone(),
        })
        .await?
        .into_inner();

    let user_ids: HashSet<String> = get_messages
        .messages
        .iter()
        .map(|it| it.user_id().into())
        .collect();

    let get_users = users_service_client
        .clone()
        .get_users(GetUsersRequest {
            user_ids: user_ids.clone().into_iter().collect(),
        })
        .await?
        .into_inner();

    Ok(AppJson(
        (get_message_messages, get_messages, get_users).try_into()?,
    ))
}

mod get_message_messages {
    use std::collections::HashMap;

    use bzd_messages_api::{
        GetMessageMessagesResponse, GetMessagesResponse, get_messages_response,
    };
    use bzd_users_api::{GetUsersResponse, get_users_response};
    use serde::{Deserialize, Serialize};

    use crate::app::error::AppError;

    #[derive(Deserialize)]
    pub struct Request {
        pub cursor_message_id: Option<String>,
    }

    #[derive(Serialize)]
    pub struct Response {
        pub messages: Vec<Message>,
        pub cursor_message_id: Option<String>,
    }

    #[derive(Serialize)]
    pub struct Message {
        pub message_id: String,
        pub text: String,
        pub user: User,
    }

    #[derive(Serialize)]
    pub struct User {
        pub user_id: String,
        pub name: String,
        pub abbr: String,
        pub color: String,
    }

    type Responses = (
        GetMessageMessagesResponse,
        GetMessagesResponse,
        GetUsersResponse,
    );

    type Users = HashMap<String, get_users_response::User>;
    type Messages = HashMap<String, get_messages_response::Message>;

    impl TryFrom<Responses> for Response {
        type Error = AppError;

        fn try_from(
            (get_message_messages, get_messages, get_users): Responses,
        ) -> Result<Self, Self::Error> {
            let users: Users = get_users
                .users
                .into_iter()
                .map(|it| (it.user_id().into(), it))
                .collect();

            let messages: Messages = get_messages
                .messages
                .into_iter()
                .map(|it| (it.message_id().into(), it))
                .collect();

            Ok(Self {
                messages: get_message_messages
                    .message_ids
                    .into_iter()
                    .map(|message_id| (message_id, &messages, &users).try_into())
                    .collect::<Result<_, _>>()?,
                cursor_message_id: get_message_messages.cursor_message_id,
            })
        }
    }

    impl TryFrom<(String, &Messages, &Users)> for Message {
        type Error = AppError;

        fn try_from(
            (message_id, messages, users): (String, &Messages, &Users),
        ) -> Result<Self, Self::Error> {
            let message = messages
                .get(&message_id)
                .ok_or(AppError::Unreachable)?
                .to_owned();

            let user = users
                .get(&message.user_id().to_string())
                .ok_or(AppError::Unreachable)?
                .to_owned();

            Ok(Self {
                message_id: message.message_id().into(),
                text: message.text().into(),
                user: User {
                    user_id: user.user_id().into(),
                    name: user.name().into(),
                    abbr: user.abbr().into(),
                    color: user.color().into(),
                },
            })
        }
    }
}
