use std::collections::HashSet;

use axum::{
    Router,
    extract::{Path, Query, State},
    routing::{delete, get, post},
};
use bzd_messages_api::{
    messages::{GetMessagesRequest, GetUserMessagesRequest},
    topics::{
        CreateTopicUserRequest, DeleteTopicUserRequest, GetTopicsRequest, GetTopicsUsersRequest,
        GetUserTopicsRequest,
    },
};
use bzd_users_api::users::{GetUserRequest, GetUserUsersRequest, GetUsersRequest};

use crate::app::{current_user::CurrentUser, error::AppError, json::AppJson, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_users))
        .route("/{user_id}", get(get_user))
        .route("/{user_id}/topics", get(get_user_topics))
        .route("/topics", post(create_user_topic))
        .route("/topics", delete(delete_user_topic))
        .route("/{user_id}/messages", get(get_user_messages))
}

async fn get_users(
    State(AppState {
        users_service_client,
        ..
    }): State<AppState>,
    user: CurrentUser,
) -> Result<AppJson<get_users::Response>, AppError> {
    let get_user_users = users_service_client
        .clone()
        .get_user_users(GetUserUsersRequest {
            user_id: user.user_id.into(),
        })
        .await?
        .into_inner();

    let get_users = users_service_client
        .clone()
        .get_users(GetUsersRequest {
            user_ids: get_user_users.user_ids.clone(),
        })
        .await?
        .into_inner();

    Ok(AppJson((get_user_users, get_users).try_into()?))
}

mod get_users {
    use std::collections::HashMap;

    use bzd_users_api::users::{GetUserUsersResponse, GetUsersResponse, get_users_response};
    use serde::Serialize;

    use crate::app::error::AppError;

    #[derive(Serialize)]
    pub struct Response {
        pub users: Vec<User>,
    }

    #[derive(Serialize)]
    pub struct User {
        pub user_id: String,
        pub name: String,
        pub abbr: String,
        pub color: String,
    }

    type Users = HashMap<String, get_users_response::User>;

    type Responses = (GetUserUsersResponse, GetUsersResponse);

    impl TryFrom<Responses> for Response {
        type Error = AppError;

        fn try_from((get_user_users, get_users): Responses) -> Result<Self, Self::Error> {
            let users: Users = get_users
                .users
                .into_iter()
                .map(|it| (it.user_id().into(), it))
                .collect();

            Ok(Self {
                users: get_user_users
                    .user_ids
                    .into_iter()
                    .map(|user_id| (user_id, &users).try_into())
                    .collect::<Result<_, _>>()?,
            })
        }
    }

    impl TryFrom<(String, &Users)> for User {
        type Error = AppError;

        fn try_from((user_id, users): (String, &Users)) -> Result<Self, Self::Error> {
            let user = users.get(&user_id).ok_or(AppError::Unreachable)?.to_owned();

            Ok(Self {
                user_id: user.user_id().into(),
                name: user.name().into(),
                abbr: user.abbr().into(),
                color: user.color().into(),
            })
        }
    }
}

async fn get_user(
    Path(user_id): Path<String>,
    State(AppState {
        users_service_client,
        ..
    }): State<AppState>,
    user: CurrentUser,
) -> Result<AppJson<get_user::Response>, AppError> {
    let get_user = users_service_client
        .clone()
        .get_user(GetUserRequest {
            user_id: user_id.into(),
        })
        .await?
        .into_inner();

    Ok(AppJson((get_user, user).try_into()?))
}

mod get_user {
    use bzd_users_api::users::GetUserResponse;
    use serde::Serialize;

    use crate::app::{current_user::CurrentUser, error::AppError};

    #[derive(Serialize)]
    pub struct Response {
        user: User,
        permissions: Permissions,
    }

    #[derive(Serialize)]
    struct Permissions {
        logout: bool,
        topics: bool,
        topics_users: bool,
    }

    #[derive(Serialize)]
    struct User {
        pub user_id: String,
        pub name: String,
        pub abbr: String,
        pub color: String,
    }

    impl TryFrom<(GetUserResponse, CurrentUser)> for Response {
        type Error = AppError;

        fn try_from(
            (get_user, current_user): (GetUserResponse, CurrentUser),
        ) -> Result<Self, Self::Error> {
            let user = get_user.user.ok_or(AppError::Unreachable)?.to_owned();

            Ok(Self {
                user: User {
                    user_id: user.user_id().into(),
                    name: user.name().into(),
                    abbr: user.abbr().into(),
                    color: user.color().into(),
                },
                // да-да, повторение, бла-бла.. пока пофиг, тут ваще не должно быть логики, чисто затычка
                permissions: Permissions {
                    logout: current_user.user_id == Some(user.user_id().into()),
                    topics: current_user.user_id == Some(user.user_id().into()),
                    topics_users: current_user.user_id != Some(user.user_id().into()),
                },
            })
        }
    }
}

async fn get_user_topics(
    Path(user_id): Path<String>,
    State(AppState {
        topics_service_client,
        ..
    }): State<AppState>,
    current_user: CurrentUser,
) -> Result<AppJson<get_user_topics::Response>, AppError> {
    let get_user_topics = topics_service_client
        .clone()
        .get_user_topics(GetUserTopicsRequest {
            user_id: user_id.into(),
        })
        .await?
        .into_inner();

    let get_topics = topics_service_client
        .clone()
        .get_topics(GetTopicsRequest {
            topic_ids: get_user_topics.topic_ids.clone(),
        })
        .await?
        .into_inner();

    let get_topics_users = topics_service_client
        .clone()
        .get_topics_users(GetTopicsUsersRequest {
            topic_ids: get_user_topics.topic_ids.clone(),
            current_user_id: current_user.user_id.clone(),
        })
        .await?
        .into_inner();

    Ok(AppJson(
        (get_topics, get_topics_users, current_user).try_into()?,
    ))
}

mod get_user_topics {
    use bzd_messages_api::topics::{
        self, GetTopicsResponse, GetTopicsUsersResponse, get_topics_users_response,
    };
    use serde::Serialize;

    use crate::app::{current_user::CurrentUser, error::AppError};

    #[derive(Serialize)]
    pub struct Response {
        topics: Vec<Topic>,
        topics_users: Vec<TopicUser>,
        permissions: Permissions,
    }

    #[derive(Serialize)]
    struct Topic {
        topic_id: String,
        title: String,
    }

    #[derive(Serialize)]
    struct TopicUser {
        topic_user_id: String,
        topic_id: String,
        user_id: String,
    }

    #[derive(Serialize)]
    pub struct Permissions {
        topics_users: bool,
    }

    type Responses = (GetTopicsResponse, GetTopicsUsersResponse, CurrentUser);

    impl TryFrom<Responses> for Response {
        type Error = AppError;

        fn try_from(
            (get_topics, get_topics_users, current_user): Responses,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                topics: get_topics.topics.iter().map(Into::into).collect(),
                topics_users: get_topics_users
                    .topics_users
                    .iter()
                    .map(Into::into)
                    .collect(),
                permissions: Permissions {
                    // эта фигня тут протекла потому что лень пока делать норм пермишны,
                    // но чтобы не тащить хардкод на мобилку, будут вот такие вот заглушки на гейте
                    topics_users: current_user.user_id.is_some(),
                },
            })
        }
    }

    impl From<&topics::Topic> for Topic {
        fn from(topic: &topics::Topic) -> Self {
            Self {
                topic_id: topic.topic_id().into(),
                title: topic.title().into(),
            }
        }
    }

    impl From<&get_topics_users_response::TopicUser> for TopicUser {
        fn from(message_topic: &get_topics_users_response::TopicUser) -> Self {
            Self {
                topic_user_id: message_topic.topic_user_id().into(),
                topic_id: message_topic.topic_id().into(),
                user_id: message_topic.user_id().into(),
            }
        }
    }
}

async fn create_user_topic(
    State(AppState {
        topics_service_client,
        ..
    }): State<AppState>,
    user: CurrentUser,
    AppJson(req): AppJson<create_user_topic::Request>,
) -> Result<AppJson<create_user_topic::Response>, AppError> {
    let res = topics_service_client
        .clone()
        .create_topic_user(CreateTopicUserRequest {
            topic_id: req.topic_id.into(),
            current_user_id: user.user_id,
        })
        .await?
        .into_inner();

    Ok(AppJson(res.try_into()?))
}

mod create_user_topic {
    use bzd_messages_api::topics::CreateTopicUserResponse;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct Request {
        pub topic_id: String,
    }

    #[derive(Serialize)]
    pub struct Response {
        pub topic_user: TopicUser,
    }

    #[derive(Serialize)]
    pub struct TopicUser {
        pub topic_user_id: String,
    }

    impl From<CreateTopicUserResponse> for Response {
        fn from(res: CreateTopicUserResponse) -> Self {
            Self {
                topic_user: TopicUser {
                    topic_user_id: res.topic_user_id().into(),
                },
            }
        }
    }
}

async fn delete_user_topic(
    State(AppState {
        topics_service_client,
        ..
    }): State<AppState>,
    user: CurrentUser,
    AppJson(req): AppJson<delete_user_topic::Request>,
) -> Result<AppJson<delete_user_topic::Response>, AppError> {
    let mut delete_topic_user_req: DeleteTopicUserRequest = req.into();
    delete_topic_user_req.current_user_id = user.user_id;

    topics_service_client
        .clone()
        .delete_topic_user(delete_topic_user_req)
        .await?;

    Ok(AppJson(delete_user_topic::Response {}))
}

mod delete_user_topic {
    use bzd_messages_api::topics::DeleteTopicUserRequest;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct Request {
        topic_user_id: String,
    }

    impl From<Request> for DeleteTopicUserRequest {
        fn from(req: Request) -> Self {
            Self {
                topic_user_id: req.topic_user_id.into(),
                current_user_id: None,
            }
        }
    }

    #[derive(Serialize)]
    pub struct Response {}
}

async fn get_user_messages(
    State(AppState {
        messages_service_client,
        users_service_client,
        ..
    }): State<AppState>,
    Path(user_id): Path<String>,
    Query(req): Query<get_user_messages::Request>,
) -> Result<AppJson<get_user_messages::Response>, AppError> {
    let get_user_messages = messages_service_client
        .clone()
        .get_user_messages(GetUserMessagesRequest {
            user_id: Some(user_id),
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

    use bzd_messages_api::messages::{
        GetMessagesResponse, GetUserMessagesResponse, get_messages_response,
    };
    use bzd_users_api::users::{GetUsersResponse, get_users_response};
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
