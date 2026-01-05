use axum::{
    Router,
    extract::{Path, State},
    routing::get,
};
use bzd_messages_api::topics::{GetTopicsRequest, GetTopicsUsersRequest, GetUserTopicsRequest};
use bzd_users_api::users::{GetUserRequest, GetUserUsersRequest, GetUsersRequest};

use crate::app::{current_user::CurrentUser, error::AppError, json::AppJson, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_users))
        .route("/{user_id}", get(get_user))
        .route("/{user_id}/topics", get(get_user_topics))
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
) -> Result<AppJson<get_user::Response>, AppError> {
    let get_user = users_service_client
        .clone()
        .get_user(GetUserRequest {
            user_id: user_id.into(),
        })
        .await?
        .into_inner();

    Ok(AppJson((get_user).try_into()?))
}

mod get_user {
    use bzd_users_api::users::GetUserResponse;
    use serde::Serialize;

    use crate::app::error::AppError;

    #[derive(Serialize)]
    pub struct Response {
        user: User,
    }

    #[derive(Serialize)]
    struct User {
        pub user_id: String,
        pub name: String,
        pub abbr: String,
        pub color: String,
    }

    impl TryFrom<GetUserResponse> for Response {
        type Error = AppError;

        fn try_from(res: GetUserResponse) -> Result<Self, Self::Error> {
            let user = res.user.ok_or(AppError::Unreachable)?.to_owned();

            Ok(Self {
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

async fn get_user_topics(
    Path(user_id): Path<String>,
    State(AppState {
        topics_service_client,
        ..
    }): State<AppState>,
    user: CurrentUser,
) -> Result<AppJson<get_user_topics::Response>, AppError> {
    let get_user_topics_res = topics_service_client
        .clone()
        .get_user_topics(GetUserTopicsRequest {
            user_id: user_id.into(),
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

    let get_topics_users_res = topics_service_client
        .clone()
        .get_topics_users(GetTopicsUsersRequest {
            topic_ids: get_user_topics_res.topic_ids.clone(),
            current_user_id: user.user_id,
        })
        .await?
        .into_inner();

    Ok(AppJson(
        (get_user_topics_res, get_topics_res, get_topics_users_res).try_into()?,
    ))
}

mod get_user_topics {
    use std::collections::HashMap;

    use bzd_messages_api::topics::{
        GetTopicsResponse, GetTopicsUsersResponse, GetUserTopicsResponse, get_topics_users_response,
    };
    use serde::Serialize;

    use crate::app::error::AppError;

    #[derive(Serialize)]
    pub struct Response {
        topics: Vec<Topic>,
    }

    #[derive(Serialize)]
    struct Topic {
        topic_id: String,
        title: String,
        topic_user: Option<TopicUser>,
    }

    #[derive(Serialize)]
    struct TopicUser {
        topic_user_id: String,
        rate: String,
        timing: String,
    }

    type Responses = (
        GetUserTopicsResponse,
        GetTopicsResponse,
        GetTopicsUsersResponse,
    );

    type TopicId = String;
    type TopicUserId = String;
    type TopicsUsers = HashMap<TopicUserId, get_topics_users_response::TopicUser>;
    type Topics = HashMap<TopicId, bzd_messages_api::topics::Topic>;

    impl TryFrom<Responses> for Response {
        type Error = AppError;

        fn try_from(
            (get_user_topics_res, get_topics_res, get_topics_users_res): Responses,
        ) -> Result<Self, Self::Error> {
            let topics_users: TopicsUsers = get_topics_users_res
                .topics_users
                .into_iter()
                .map(|it| (it.topic_id().into(), it))
                .collect();

            let topics: Topics = get_topics_res
                .topics
                .into_iter()
                .map(|it| (it.topic_id().into(), it))
                .collect();

            Ok(Self {
                topics: get_user_topics_res
                    .topic_ids
                    .into_iter()
                    .map(|it| (it, &topics, &topics_users).try_into())
                    .collect::<Result<_, _>>()?,
            })
        }
    }

    impl TryFrom<(TopicId, &Topics, &TopicsUsers)> for Topic {
        type Error = AppError;

        fn try_from(
            (topic_id, topics, topics_users): (TopicId, &Topics, &TopicsUsers),
        ) -> Result<Self, Self::Error> {
            let topic = topics
                .get(&topic_id)
                .ok_or(AppError::Unreachable)?
                .to_owned();

            Ok(Self {
                topic_id: topic.topic_id().into(),
                title: topic.title().into(),
                topic_user: topics_users.get(topic.topic_id()).map(|it| TopicUser {
                    topic_user_id: it.topic_user_id().into(),
                    rate: it.rate().as_str_name().into(),
                    timing: it.timing().as_str_name().into(),
                }),
            })
        }
    }
}
