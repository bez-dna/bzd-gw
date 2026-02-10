use std::collections::HashSet;

use axum::{
    Router,
    extract::{Path, Query, State},
    routing::{delete, get, post},
};
use bzd_feeds_api::feeds::GetUserEntriesRequest;
use bzd_messages_api::{
    messages::{
        CreateMessageRequest, CreateMessageTopicRequest, DeleteMessageTopicRequest,
        GetMessageMessagesRequest, GetMessageRequest, GetMessagesRequest, GetMessagesTopicsRequest,
        GetStreamsRequest,
    },
    topics::{GetTopicsRequest, GetUserTopicsRequest},
};
use bzd_users_api::users::GetUsersRequest;

use crate::app::{current_user::CurrentUser, error::AppError, json::AppJson, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_message))
        .route("/", get(get_feed_messages))
        .route("/{message_id}", get(get_message))
        .route("/{message_id}/messages", get(get_message_messages))
        .route("/{message_id}/topics", get(get_message_topics))
        .route("/topics", post(create_message_topic))
        .route("/topics", delete(delete_message_topic))
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
    use bzd_messages_api::messages::{CreateMessageRequest, CreateMessageResponse};
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct Request {
        pub text: String,
        pub code: String,
        pub message_id: Option<String>,
    }

    impl From<Request> for CreateMessageRequest {
        fn from(req: Request) -> Self {
            Self {
                text: Some(req.text),
                current_user_id: None,
                code: Some(req.code),
                message_id: req.message_id,
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

async fn get_feed_messages(
    State(AppState {
        messages_service_client,
        users_service_client,
        feeds,
        ..
    }): State<AppState>,
    user: CurrentUser,
    Query(req): Query<get_feed_messages::Request>,
) -> Result<AppJson<get_feed_messages::Response>, AppError> {
    // sleep(Duration::from_secs(1)).await;

    let get_user_entries_res = feeds
        .clone()
        .get_user_entries(GetUserEntriesRequest {
            user_id: user.user_id.into(),
            cursor_entry_id: req.cursor_message_id.into(),
        })
        .await?
        .into_inner();

    let get_messages_res = messages_service_client
        .clone()
        .get_messages(GetMessagesRequest {
            message_ids: get_user_entries_res.message_ids.clone(),
        })
        .await?
        .into_inner();

    let user_ids: HashSet<String> = get_messages_res
        .messages
        .iter()
        .map(|it| it.user_id().into())
        .collect();

    let get_users_res = users_service_client
        .clone()
        .get_users(GetUsersRequest {
            user_ids: user_ids.clone().into_iter().collect(),
        })
        .await?
        .into_inner();

    Ok(AppJson(
        (get_user_entries_res, get_messages_res, get_users_res).try_into()?,
    ))
}

mod get_feed_messages {
    use std::collections::HashMap;

    use bzd_feeds_api::feeds::GetUserEntriesResponse;
    use bzd_messages_api::messages::{GetMessagesResponse, get_messages_response};
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
        GetUserEntriesResponse,
        GetMessagesResponse,
        GetUsersResponse,
    );

    type Users = HashMap<String, get_users_response::User>;
    type Messages = HashMap<String, get_messages_response::Message>;

    impl TryFrom<Responses> for Response {
        type Error = AppError;

        fn try_from(
            (get_user_entries, get_messages, get_users): Responses,
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
                messages: get_user_entries
                    .message_ids
                    .into_iter()
                    .map(|message_id| (message_id, &messages, &users).try_into())
                    .collect::<Result<_, _>>()?,
                cursor_message_id: get_user_entries.cursor_entry_id,
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
    user: CurrentUser,
) -> Result<AppJson<get_message::Response>, AppError> {
    let get_message = messages_service_client
        .clone()
        .get_message(GetMessageRequest {
            message_id: message_id.into(),
            current_user_id: user.user_id,
        })
        .await?
        .into_inner();

    Ok(AppJson(get_message.try_into()?))
}

mod get_message {
    use bzd_messages_api::messages::GetMessageResponse;
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
        permissions: Permissions,
    }

    #[derive(Serialize)]
    struct Permissions {
        topics: bool,
    }

    impl TryFrom<GetMessageResponse> for Response {
        type Error = AppError;

        fn try_from(get_message: GetMessageResponse) -> Result<Self, Self::Error> {
            let message = get_message.message.ok_or(AppError::Unreachable)?;

            Ok(Self {
                message: Message {
                    message_id: message.message_id().into(),
                    text: message.text().into(),
                    permissions: Permissions {
                        topics: message.permissions.is_some_and(|p| p.topics()),
                    },
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

    let get_streams = messages_service_client
        .clone()
        .get_streams(GetStreamsRequest {
            message_ids: get_message_messages.message_ids.clone(),
        })
        .await?
        .into_inner();

    let user_ids: HashSet<String> = get_messages
        .messages
        .iter()
        .map(|it| it.user_id().into())
        .chain(
            get_streams
                .streams
                .iter()
                .flat_map(|it| it.user_ids.clone())
                .collect::<Vec<_>>(),
        )
        .collect();

    let get_users = users_service_client
        .clone()
        .get_users(GetUsersRequest {
            user_ids: user_ids.clone().into_iter().collect(),
        })
        .await?
        .into_inner();

    Ok(AppJson(
        (get_message_messages, get_messages, get_users, get_streams).try_into()?,
    ))
}

mod get_message_messages {
    use std::collections::HashMap;

    use bzd_messages_api::messages::{
        GetMessageMessagesResponse, GetMessagesResponse, GetStreamsResponse, get_messages_response,
        get_streams_response,
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
        pub stream: Option<Stream>,
    }

    #[derive(Serialize)]
    pub struct User {
        pub user_id: String,
        pub name: String,
        pub abbr: String,
        pub color: String,
    }

    #[derive(Serialize)]
    pub struct Stream {
        pub stream_id: String,
        pub message_id: String,
        pub text: String,
        pub messages_count: i64,
        pub users: Vec<User>,
    }

    type Responses = (
        GetMessageMessagesResponse,
        GetMessagesResponse,
        GetUsersResponse,
        GetStreamsResponse,
    );

    type Users = HashMap<String, get_users_response::User>;
    type Messages = HashMap<String, get_messages_response::Message>;
    type Streams = HashMap<String, get_streams_response::Stream>;

    impl TryFrom<Responses> for Response {
        type Error = AppError;

        fn try_from(
            (get_message_messages, get_messages, get_users, get_streams): Responses,
        ) -> Result<Self, Self::Error> {
            let users: Users = get_users
                .users
                .into_iter()
                .map(|it| (it.user_id().into(), it))
                .collect();

            let streams: Streams = get_streams
                .streams
                .into_iter()
                .map(|it| (it.message_id().into(), it))
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
                    .map(|message_id| (message_id, &messages, &users, &streams).try_into())
                    .collect::<Result<_, _>>()?,
                cursor_message_id: get_message_messages.cursor_message_id,
            })
        }
    }

    impl TryFrom<(String, &Messages, &Users, &Streams)> for Message {
        type Error = AppError;

        fn try_from(
            (message_id, messages, users, streams): (String, &Messages, &Users, &Streams),
        ) -> Result<Self, Self::Error> {
            let message = messages.get(&message_id).ok_or(AppError::Unreachable)?;

            let user = users
                .get(&message.user_id().to_string())
                .ok_or(AppError::Unreachable)?;

            let stream = streams.get(&message.message_id().to_string());

            Ok(Self {
                message_id: message.message_id().into(),
                text: message.text().into(),
                user: User {
                    user_id: user.user_id().into(),
                    name: user.name().into(),
                    abbr: user.abbr().into(),
                    color: user.color().into(),
                },
                stream: stream
                    .map(|stream| (stream, users).try_into())
                    .transpose()?,
            })
        }
    }

    impl TryFrom<(&get_streams_response::Stream, &Users)> for Stream {
        type Error = AppError;

        fn try_from(
            (stream, users): (&get_streams_response::Stream, &Users),
        ) -> Result<Self, Self::Error> {
            Ok(Stream {
                stream_id: stream.stream_id().into(),
                message_id: stream.message_id().into(),
                text: stream.text().into(),
                messages_count: stream.messages_count(),
                users: stream
                    .user_ids
                    .iter()
                    .map(|user_id| (user_id, users).try_into())
                    .collect::<Result<_, _>>()?,
            })
        }
    }

    impl TryFrom<(&String, &Users)> for User {
        type Error = AppError;

        fn try_from((user_id, users): (&String, &Users)) -> Result<Self, Self::Error> {
            let user = users.get(user_id).ok_or(AppError::Unreachable)?;

            Ok(User {
                user_id: user.user_id().into(),
                name: user.name().into(),
                abbr: user.abbr().into(),
                color: user.color().into(),
            })
        }
    }
}

async fn get_message_topics(
    Path(message_id): Path<String>,
    State(AppState {
        topics_service_client,
        messages_service_client,
        ..
    }): State<AppState>,
    user: CurrentUser,
) -> Result<AppJson<get_message_topics::Response>, AppError> {
    let get_messages_topics = messages_service_client
        .clone()
        .get_messages_topics(GetMessagesTopicsRequest {
            message_ids: vec![message_id],
        })
        .await?
        .into_inner();

    let get_user_topics = topics_service_client
        .clone()
        .get_user_topics(GetUserTopicsRequest {
            user_id: user.user_id,
        })
        .await?
        .into_inner();

    let get_topics = topics_service_client
        .clone()
        .get_topics(GetTopicsRequest {
            topic_ids: get_user_topics.topic_ids,
        })
        .await?
        .into_inner();

    Ok(AppJson((get_topics, get_messages_topics).into()))
}

mod get_message_topics {
    use bzd_messages_api::{
        messages::{GetMessagesTopicsResponse, get_messages_topics_response},
        topics::{self, GetTopicsResponse},
    };
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct Response {
        pub topics: Vec<Topic>,
        pub messages_topics: Vec<MessageTopic>,
    }

    #[derive(Serialize)]
    pub struct Topic {
        pub topic_id: String,
        pub title: String,
    }

    #[derive(Serialize)]
    pub struct MessageTopic {
        pub message_topic_id: String,
        pub topic_id: String,
        pub message_id: String,
    }

    type Responses = (GetTopicsResponse, GetMessagesTopicsResponse);

    impl From<Responses> for Response {
        fn from((get_topics, get_messages_topics): Responses) -> Self {
            Self {
                topics: get_topics.topics.iter().map(Into::into).collect(),
                messages_topics: get_messages_topics
                    .messages_topics
                    .iter()
                    .map(Into::into)
                    .collect(),
            }
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

    impl From<&get_messages_topics_response::MessageTopic> for MessageTopic {
        fn from(message_topic: &get_messages_topics_response::MessageTopic) -> Self {
            Self {
                message_topic_id: message_topic.message_topic_id().into(),
                topic_id: message_topic.topic_id().into(),
                message_id: message_topic.message_id().into(),
            }
        }
    }
}

async fn create_message_topic(
    State(AppState {
        messages_service_client,
        ..
    }): State<AppState>,
    user: CurrentUser,
    AppJson(req): AppJson<create_message_topic::Request>,
) -> Result<AppJson<create_message_topic::Response>, AppError> {
    let res = messages_service_client
        .clone()
        .create_message_topic(CreateMessageTopicRequest {
            current_user_id: user.user_id,
            message_id: Some(req.message_id),
            topic_id: Some(req.topic_id),
        })
        .await?
        .into_inner();

    Ok(AppJson(res.into()))
}

mod create_message_topic {
    use bzd_messages_api::messages::CreateMessageTopicResponse;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct Request {
        pub message_id: String,
        pub topic_id: String,
    }

    #[derive(Serialize)]
    pub struct Response {
        pub message_topic: MessageTopic,
    }

    #[derive(Serialize)]
    pub struct MessageTopic {
        pub message_topic_id: String,
    }

    impl From<CreateMessageTopicResponse> for Response {
        fn from(res: CreateMessageTopicResponse) -> Self {
            Self {
                message_topic: MessageTopic {
                    message_topic_id: res.message_topic_id().into(),
                },
            }
        }
    }
}

async fn delete_message_topic(
    State(AppState {
        messages_service_client,
        ..
    }): State<AppState>,
    user: CurrentUser,
    AppJson(req): AppJson<delete_message_topic::Request>,
) -> Result<AppJson<delete_message_topic::Response>, AppError> {
    let res = messages_service_client
        .clone()
        .delete_message_topic(DeleteMessageTopicRequest {
            current_user_id: user.user_id,
            message_topic_id: Some(req.message_topic_id),
        })
        .await?
        .into_inner();

    Ok(AppJson(res.into()))
}

mod delete_message_topic {
    use bzd_messages_api::messages::DeleteMessageTopicResponse;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct Request {
        pub message_topic_id: String,
    }

    #[derive(Serialize)]
    pub struct Response {}

    impl From<DeleteMessageTopicResponse> for Response {
        fn from(_: DeleteMessageTopicResponse) -> Self {
            Self {}
        }
    }
}
