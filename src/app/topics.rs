use axum::{
    Router,
    extract::State,
    routing::{get, post},
};
use bzd_messages_api::topics::{
    CreateTopicRequest, GetEmojisRequest, GetTopicRequest, GetTopicsRequest, GetUserTopicsRequest,
};

use crate::app::{current_user::CurrentUser, error::AppError, json::AppJson, state::AppState};

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
    user: CurrentUser,
) -> Result<AppJson<get_topics::Response>, AppError> {
    let get_user_topics = topics_service_client
        .clone()
        .get_user_topics(GetUserTopicsRequest {
            user_id: user.user_id.clone(),
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

    let get_emojis = topics_service_client
        .clone()
        .get_emojis(GetEmojisRequest::default())
        .await?
        .into_inner();

    Ok(AppJson(
        (get_topics, get_user_topics, get_emojis, user).try_into()?,
    ))
}

mod get_topics {
    use std::collections::HashMap;

    use bzd_messages_api::topics::{
        GetEmojisResponse, GetTopicsResponse, GetUserTopicsResponse, get_emojis_response,
    };
    use serde::Serialize;

    use crate::app::{current_user::CurrentUser, error::AppError};

    #[derive(Serialize)]
    pub struct Response {
        pub topics: Vec<Topic>,
        pub emojis: Vec<Emoji>,
        pub permissions: Permissions,
    }

    #[derive(Serialize)]
    pub struct Topic {
        pub topic_id: String,
        pub title: String,
    }

    #[derive(Serialize)]
    pub struct Emoji {
        pub title: String,
        pub code: String,
    }

    #[derive(Serialize)]
    pub struct Permissions {
        topics: bool,
    }

    type Responses = (
        GetTopicsResponse,
        GetUserTopicsResponse,
        GetEmojisResponse,
        CurrentUser,
    );

    type TopicId = String;
    type Topics = HashMap<TopicId, bzd_messages_api::topics::Topic>;

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

        fn try_from(
            (get_topics, get_user_topics, get_emojis, current_user): Responses,
        ) -> Result<Self, Self::Error> {
            let topics: Topics = get_topics
                .topics
                .into_iter()
                .map(|it| (it.topic_id().into(), it))
                .collect();

            Ok(Self {
                topics: get_user_topics
                    .topic_ids
                    .into_iter()
                    .map(|it| (it, &topics).try_into())
                    .collect::<Result<_, _>>()?,
                emojis: get_emojis.emojis.into_iter().map(Into::into).collect(),
                permissions: Permissions {
                    topics: current_user.user_id.is_some(),
                },
            })
        }
    }

    impl From<get_emojis_response::Emoji> for Emoji {
        fn from(emoji: get_emojis_response::Emoji) -> Self {
            Self {
                title: emoji.title().into(),
                code: emoji.code().into(),
            }
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
    req.current_user_id = user.user_id.clone();

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
    use bzd_messages_api::topics::{CreateTopicRequest, GetTopicResponse};
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
                current_user_id: None,
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
