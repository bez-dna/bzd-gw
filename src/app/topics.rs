use axum::{
    Router,
    extract::State,
    routing::{delete, get, post},
};
use bzd_messages_api::{
    CreateTopicRequest, CreateTopicUserRequest, DeleteTopicUserRequest, GetTopicRequest,
    GetTopicsRequest,
};

use crate::app::{error::AppError, json::AppJson, state::AppState, user::AppUser};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_topics))
        .route("/", post(create_topic))
        .route("/users", post(create_topic_user))
        .route("/users", delete(delete_topic_user))
}

async fn get_topics(
    State(AppState {
        topics_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
) -> Result<AppJson<get_topics::Response>, AppError> {
    let req = GetTopicsRequest {
        user_ids: vec![user.user_id],
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
    req.user_id = Some(user.user_id.clone().into());

    let create_topic_response = topics_service_client
        .clone()
        .create_topic(req)
        .await?
        .into_inner();

    /*
    Тут есть нюанс, мы могли бы вернуть целый объект в методе create_topic и не делать второй вызов get_topic
    Это сделано специально, потому что я не хочу разделять модели логически (не в коде, где можно переиспользовать
    и в прото, и в коде), а "бизнесово". Я буквально с точки зрения проектирования системы гарантирую что получение
    сущности у меня в одном методе get_topic, и да, пока нет проблем перформанса это будет так.
    Когда ручке get_topic будет плохо из-за того что её вызывают после create_topic (где наступление этого события
    стремится за горизонит событий), это будет очень дешево исправить, просто вернув целиком модель.
     */

    let res = topics_service_client
        .clone()
        .get_topic(GetTopicRequest {
            topic_id: create_topic_response.topic_id,
            user_id: Some(user.user_id.into()),
        })
        .await?
        .into_inner();

    Ok(AppJson(res.try_into()?))
}

mod create_topic {
    use bzd_messages_api::{CreateTopicRequest, GetTopicResponse};
    use serde::{Deserialize, Serialize};

    use crate::app::error::AppError;

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
        pub title: String,
    }

    impl TryFrom<GetTopicResponse> for Response {
        type Error = AppError;

        fn try_from(res: GetTopicResponse) -> Result<Self, Self::Error> {
            let topic = res.topic.ok_or(AppError::Internal)?;

            Ok(Self {
                topic: Topic {
                    topic_id: topic.topic_id().into(),
                    title: topic.title().into(),
                },
            })
        }
    }
}

async fn create_topic_user(
    State(AppState {
        topics_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
    AppJson(req): AppJson<create_topic_user::Request>,
) -> Result<AppJson<create_topic_user::Response>, AppError> {
    let res = topics_service_client
        .clone()
        .create_topic_user(CreateTopicUserRequest {
            topic_id: req.topic_id.into(),
            user_id: user.user_id.into(),
        })
        .await?
        .into_inner();

    Ok(AppJson(res.try_into()?))
}

mod create_topic_user {
    use bzd_messages_api::CreateTopicUserResponse;
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

async fn delete_topic_user(
    State(AppState {
        topics_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
    AppJson(req): AppJson<delete_topic_user::Request>,
) -> Result<AppJson<delete_topic_user::Response>, AppError> {
    let mut delete_topic_user_req: DeleteTopicUserRequest = req.into();
    delete_topic_user_req.user_id = user.user_id.into();

    topics_service_client
        .clone()
        .delete_topic_user(delete_topic_user_req)
        .await?;

    Ok(AppJson(delete_topic_user::Response {}))
}

mod delete_topic_user {
    use bzd_messages_api::DeleteTopicUserRequest;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct Request {
        pub topic_user_id: String,
    }

    impl From<Request> for DeleteTopicUserRequest {
        fn from(req: Request) -> Self {
            Self {
                topic_user_id: req.topic_user_id.into(),
                user_id: None,
            }
        }
    }

    #[derive(Serialize)]
    pub struct Response {}
}
