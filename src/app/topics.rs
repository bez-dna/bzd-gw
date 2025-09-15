use axum::{
    Router,
    extract::State,
    routing::{get, post},
};
use bzd_messages_api::{CreateTopicRequest, GetTopicsRequest};

use crate::app::{error::AppError, json::AppJson, state::AppState, user::AppUser};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_topics))
        .route("/", post(create_topic))
}

async fn get_topics(
    State(AppState {
        topics_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
) -> Result<AppJson<get_topics::Response>, AppError> {
    let req = GetTopicsRequest {
        user_id: Some(user.user_id),
    };

    let res = topics_service_client
        .clone()
        .get_topics(req)
        .await?
        .into_inner();

    Ok(AppJson(res.into()))
}

mod get_topics {
    use bzd_messages_api::{GetTopicsResponse, get_topics_response};
    use serde::Serialize;

    #[derive(Serialize)]
    pub struct Response {
        pub topics: Vec<Topic>,
    }

    #[derive(Serialize)]
    pub struct Topic {
        pub topic_id: String,
        pub title: String,
    }

    impl From<GetTopicsResponse> for Response {
        fn from(res: GetTopicsResponse) -> Self {
            Self {
                topics: res.topics.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl From<get_topics_response::Topic> for Topic {
        fn from(topic: get_topics_response::Topic) -> Self {
            Self {
                topic_id: topic.topic_id().into(),
                title: topic.title().into(),
            }
        }
    }
}

async fn create_topic(
    State(AppState {
        topics_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
    AppJson(data): AppJson<create_topic::Request>,
) -> Result<AppJson<create_topic::Response>, AppError> {
    let mut req: CreateTopicRequest = data.into();
    req.user_id = Some(user.user_id.into());

    let res = topics_service_client
        .clone()
        .create_topic(req)
        .await?
        .into_inner();

    Ok(AppJson(res.into()))
}

mod create_topic {
    use bzd_messages_api::{CreateTopicRequest, CreateTopicResponse};
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct Request {
        pub title: String,
    }

    impl From<Request> for CreateTopicRequest {
        fn from(req: Request) -> Self {
            Self {
                title: Some(req.title),
                user_id: None,
            }
        }
    }

    #[derive(Serialize)]
    pub struct Response {
        pub topic: Topic,
    }

    #[derive(Serialize)]
    pub struct Topic {
        pub topic_id: String,
    }

    impl From<CreateTopicResponse> for Response {
        fn from(res: CreateTopicResponse) -> Self {
            Self {
                topic: Topic {
                    topic_id: res.topic_id().into(),
                },
            }
        }
    }
}
