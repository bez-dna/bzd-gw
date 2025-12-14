use axum::{
    Router,
    extract::State,
    routing::{delete, get, patch, post},
};
use bzd_messages_api::{
    CreateTopicRequest, CreateTopicUserRequest, DeleteTopicUserRequest, GetTopicRequest,
    GetTopicsRequest, GetUserTopicsRequest, UpdateTopicUserRequest,
};

use crate::app::{current_user::CurrentUser, error::AppError, json::AppJson, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_topics))
        .route("/", post(create_topic))
        .route("/users", post(create_topic_user))
        .route("/users", patch(update_topic_user))
        .route("/users", delete(delete_topic_user))
}

async fn get_topics(
    State(AppState {
        topics_service_client,
        ..
    }): State<AppState>,
    user: CurrentUser,
) -> Result<AppJson<get_topics::Response>, AppError> {
    let get_user_topics_res = topics_service_client
        .clone()
        .get_user_topics(GetUserTopicsRequest {
            user_id: user.user_id,
        })
        .await?
        .into_inner();

    let get_topics_res = topics_service_client
        .clone()
        .get_topics(GetTopicsRequest {
            topic_ids: get_user_topics_res.topic_ids.clone(),
        })
        .await?
        .into_inner();

    Ok(AppJson((get_user_topics_res, get_topics_res).try_into()?))
}

mod get_topics {
    use std::collections::HashMap;

    use bzd_messages_api::{GetTopicsResponse, GetUserTopicsResponse};
    use serde::Serialize;

    use crate::app::error::AppError;

    #[derive(Serialize)]
    pub struct Response {
        pub topics: Vec<Topic>,
    }

    #[derive(Serialize)]
    pub struct Topic {
        pub topic_id: String,
        pub title: String,
    }

    type Responses = (GetUserTopicsResponse, GetTopicsResponse);

    type TopicId = String;
    type Topics = HashMap<TopicId, bzd_messages_api::Topic>;

    impl TryFrom<(TopicId, &Topics)> for Topic {
        type Error = AppError;

        fn try_from((topic_id, topics): (String, &Topics)) -> Result<Self, Self::Error> {
            let topic = topics
                .get(&topic_id)
                .ok_or(AppError::Unreachable)?
                .to_owned();

            Ok(Self {
                topic_id: topic.topic_id().into(),
                title: topic.title().into(),
            })
        }
    }

    impl TryFrom<Responses> for Response {
        type Error = AppError;

        fn try_from((get_user_topics_res, get_topics_res): Responses) -> Result<Self, Self::Error> {
            let topics: Topics = get_topics_res
                .topics
                .into_iter()
                .map(|it| (it.topic_id().into(), it))
                .collect();

            Ok(Self {
                topics: get_user_topics_res
                    .topic_ids
                    .into_iter()
                    .map(|it| (it, &topics).try_into())
                    .collect::<Result<_, _>>()?,
            })
        }
    }
}

async fn create_topic(
    State(AppState {
        topics_service_client,
        ..
    }): State<AppState>,
    user: CurrentUser,
    AppJson(data): AppJson<create_topic::Request>,
) -> Result<AppJson<create_topic::Response>, AppError> {
    let mut req: CreateTopicRequest = data.into();
    req.user_id = user.user_id.clone();

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
            let topic = res.topic.ok_or(AppError::Unreachable)?;

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
    user: CurrentUser,
    AppJson(req): AppJson<create_topic_user::Request>,
) -> Result<AppJson<create_topic_user::Response>, AppError> {
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

async fn update_topic_user(
    State(AppState {
        topics_service_client,
        ..
    }): State<AppState>,
    user: CurrentUser,
    AppJson(req): AppJson<update_topic_user::Request>,
) -> Result<AppJson<update_topic_user::Response>, AppError> {
    let mut update_topic_user_req: UpdateTopicUserRequest = req.try_into()?;
    update_topic_user_req.current_user_id = user.user_id.into();

    topics_service_client
        .clone()
        .update_topic_user(update_topic_user_req)
        .await?;

    Ok(AppJson(update_topic_user::Response {}))
}

mod update_topic_user {
    use bzd_messages_api::{Rate, Timing, UpdateTopicUserRequest};
    use serde::{Deserialize, Serialize};

    use crate::app::error::AppError;

    #[derive(Deserialize, Debug)]
    pub struct Request {
        topic_user_id: String,
        rate: String,
        timing: String,
    }

    impl TryFrom<Request> for UpdateTopicUserRequest {
        type Error = AppError;

        fn try_from(req: Request) -> Result<Self, Self::Error> {
            let rate = Rate::from_str_name(&req.rate).ok_or(AppError::Common)?;
            let timing = Timing::from_str_name(&req.timing).ok_or(AppError::Common)?;

            Ok(Self {
                current_user_id: None,
                topic_user_id: req.topic_user_id.into(),
                timing: Some(timing.into()),
                rate: Some(rate.into()),
            })
        }
    }

    #[derive(Serialize)]
    pub struct Response {}
}

async fn delete_topic_user(
    State(AppState {
        topics_service_client,
        ..
    }): State<AppState>,
    user: CurrentUser,
    AppJson(req): AppJson<delete_topic_user::Request>,
) -> Result<AppJson<delete_topic_user::Response>, AppError> {
    let mut delete_topic_user_req: DeleteTopicUserRequest = req.into();
    delete_topic_user_req.current_user_id = user.user_id;

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
