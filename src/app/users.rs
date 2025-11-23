use std::collections::HashSet;

use axum::{
    Router,
    extract::{Path, State},
    routing::get,
};
use bzd_messages_api::{GetTopicsUsersRequest, GetUserTopicsRequest};
use bzd_users_api::{GetUserRequest, GetUserUsersRequest, GetUsersRequest};

use crate::app::{error::AppError, json::AppJson, state::AppState, user::AppUser};

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
    user: AppUser,
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

    use bzd_users_api::{GetUserUsersResponse, GetUsersResponse, get_users_response};
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
    use bzd_users_api::GetUserResponse;
    use serde::Serialize;

    use crate::app::error::AppError;

    #[derive(Serialize)]
    pub struct Response {
        user: User,
        topics: Vec<Topic>,
    }

    #[derive(Serialize)]
    struct User {
        pub user_id: String,
        pub name: String,
        pub abbr: String,
        pub color: String,
    }

    #[derive(Serialize)]
    struct Topic {
        pub topic_id: String,
        pub title: String,
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
                topics: vec![],
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
    user: Option<AppUser>,
) -> Result<AppJson<get_user_topics::Response>, AppError> {
    let get_user_topics = topics_service_client
        .clone()
        .get_user_topics(GetUserTopicsRequest {
            user_id: user_id.into(),
        })
        .await?
        .into_inner();

    let topic_ids: HashSet<String> = get_user_topics
        .topics
        .iter()
        .map(|it| it.topic_id().into())
        .collect();

    let get_topics_users = topics_service_client
        .clone()
        .get_topics_users(GetTopicsUsersRequest {
            topic_ids: topic_ids.into_iter().collect(),
            user_id: if let Some(user) = user {
                user.user_id.into()
            } else {
                None
            },
        })
        .await?
        .into_inner();

    Ok(AppJson((get_user_topics, get_topics_users).try_into()?))
}

mod get_user_topics {
    use std::collections::HashMap;

    use bzd_messages_api::{
        GetTopicsUsersResponse, GetUserTopicsResponse, get_topics_users_response,
        get_user_topics_response,
    };
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
        pub topic_user: Option<TopicUser>,
    }

    #[derive(Serialize)]
    pub struct TopicUser {
        pub topic_user_id: String,
    }

    type Responses = (GetUserTopicsResponse, GetTopicsUsersResponse);
    type TopicsUsers = HashMap<String, get_topics_users_response::TopicUser>;

    impl TryFrom<Responses> for Response {
        type Error = AppError;

        fn try_from((get_user_topics, get_topics_users): Responses) -> Result<Self, Self::Error> {
            let topics_users: TopicsUsers = get_topics_users
                .topics_users
                .into_iter()
                .map(|it| (it.topic_id().into(), it))
                .collect();

            Ok(Self {
                topics: get_user_topics
                    .topics
                    .into_iter()
                    .map(|it| (it, &topics_users).try_into())
                    .collect::<Result<_, _>>()?,
            })
        }
    }

    impl TryFrom<(get_user_topics_response::Topic, &TopicsUsers)> for Topic {
        type Error = AppError;

        fn try_from(
            (topic, topics_users): (get_user_topics_response::Topic, &TopicsUsers),
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                topic_id: topic.topic_id().into(),
                title: topic.title().into(),
                topic_user: topics_users.get(topic.topic_id()).map(|it| TopicUser {
                    topic_user_id: it.topic_user_id().into(),
                }),
            })
        }
    }
}
