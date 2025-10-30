use std::collections::HashSet;

use axum::{
    Router,
    extract::{Path, State},
    routing::get,
};
use bzd_messages_api::{GetTopicsRequest, GetTopicsUsersRequest};
use bzd_users_api::{GetSourceRequest, GetSourcesRequest, GetUserRequest, GetUsersRequest};

use crate::app::{error::AppError, json::AppJson, state::AppState, user::AppUser};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_users))
        .route("/{user_id}", get(get_user))
}

async fn get_users(
    State(AppState {
        sources_service_client,
        users_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
) -> Result<AppJson<get_users::Response>, AppError> {
    let req = GetSourcesRequest {
        user_id: Some(user.user_id.into()),
    };

    let get_sources_response = sources_service_client
        .clone()
        .get_sources(req)
        .await?
        .into_inner();

    let user_ids: HashSet<String> = get_sources_response
        .contacts
        .iter()
        .map(|it| it.contact_user_id().into())
        .chain(
            get_sources_response
                .sources
                .iter()
                .map(|it| it.source_user_id().into()),
        )
        .collect();

    let get_users_response = users_service_client
        .clone()
        .get_users(GetUsersRequest {
            user_ids: user_ids.into_iter().collect(),
        })
        .await?
        .into_inner();

    Ok(AppJson(
        (get_sources_response, get_users_response).try_into()?,
    ))
}

mod get_users {
    use std::collections::HashMap;

    use bzd_users_api::{
        GetSourcesResponse, GetUsersResponse, get_sources_response, get_users_response,
    };
    use serde::Serialize;

    use crate::app::error::AppError;

    #[derive(Serialize)]
    pub struct Response {
        pub sources: Sources,
        pub contacts: Contacts,
    }

    type Sources = Vec<Source>;

    #[derive(Serialize)]
    pub struct Source {
        pub source_id: String,
        pub user: User,
    }

    type Contacts = Vec<Contact>;

    #[derive(Serialize)]
    pub struct Contact {
        pub contact_id: String,
        pub contact_name: String,
        pub user: User,
    }

    #[derive(Serialize)]
    pub struct User {
        pub user_id: String,
        pub name: String,
        pub phone: String,
        pub abbr: String,
        pub color: String,
    }

    type Users = HashMap<String, get_users_response::User>;

    impl TryFrom<(GetSourcesResponse, GetUsersResponse)> for Response {
        type Error = AppError;

        fn try_from(
            (get_sources_response, get_users_response): (GetSourcesResponse, GetUsersResponse),
        ) -> Result<Self, Self::Error> {
            let users: Users = get_users_response
                .users
                .into_iter()
                .map(|user| (user.user_id().into(), user))
                .collect();

            let contacts: Contacts = get_sources_response
                .contacts
                .into_iter()
                .map(|contact| (contact, &users).try_into())
                .collect::<Result<_, _>>()?;

            let sources: Sources = get_sources_response
                .sources
                .into_iter()
                .map(|source| (source, &users).try_into())
                .collect::<Result<_, _>>()?;

            Ok(Self { contacts, sources })
        }
    }

    impl TryFrom<(get_sources_response::Contact, &Users)> for Contact {
        type Error = AppError;

        fn try_from(
            (contact, users): (get_sources_response::Contact, &Users),
        ) -> Result<Self, Self::Error> {
            let user = users
                .get(&contact.contact_user_id().to_string())
                .ok_or(AppError::Internal)?
                .to_owned();

            Ok(Self {
                contact_id: contact.contact_id().into(),
                contact_name: contact.name().into(),

                user: User {
                    user_id: user.user_id().into(),
                    name: user.name().into(),
                    phone: user.phone().into(),
                    abbr: user.abbr().into(),
                    color: user.color().into(),
                },
            })
        }
    }

    impl TryFrom<(get_sources_response::Source, &Users)> for Source {
        type Error = AppError;

        fn try_from(
            (source, users): (get_sources_response::Source, &Users),
        ) -> Result<Self, Self::Error> {
            let user = users
                .get(&source.source_user_id().to_string())
                .ok_or(AppError::Internal)?
                .to_owned();

            Ok(Self {
                source_id: source.source_id().into(),

                user: User {
                    user_id: user.user_id().into(),
                    name: user.name().into(),
                    phone: user.phone().into(),
                    abbr: user.abbr().into(),
                    color: user.color().into(),
                },
            })
        }
    }
}

async fn get_user(
    State(AppState {
        sources_service_client,
        users_service_client,
        topics_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
    Path(user_id): Path<String>,
) -> Result<AppJson<get_user::Response>, AppError> {
    let get_source_response = sources_service_client
        .clone()
        .get_source(GetSourceRequest {
            source_user_id: user_id.into(),
            user_id: user.user_id.into(),
        })
        .await?
        .into_inner();

    let source_user_id = get_source_response
        .source
        .clone()
        .map(|it| it.source_user_id().to_owned())
        .ok_or(AppError::Internal)?;

    let get_user_response = users_service_client
        .clone()
        .get_user(GetUserRequest {
            user_id: Some(source_user_id.clone().into()),
        })
        .await?
        .into_inner();

    let get_topics_response = topics_service_client
        .clone()
        .get_topics(GetTopicsRequest {
            user_id: Some(source_user_id.clone().into()),
        })
        .await?
        .into_inner();

    let topic_ids: HashSet<String> = get_topics_response
        .topics
        .iter()
        .map(|it| it.topic_id().into())
        .collect();

    let get_topics_users_response = topics_service_client
        .clone()
        .get_topics_users(GetTopicsUsersRequest {
            topic_ids: topic_ids.into_iter().collect(),
            user_id: Some(source_user_id.into()),
        })
        .await?
        .into_inner();

    Ok(AppJson(
        (
            get_source_response,
            get_user_response,
            get_topics_response,
            get_topics_users_response,
        )
            .try_into()?,
    ))
}

mod get_user {
    use std::collections::HashMap;

    use bzd_messages_api::{GetTopicsResponse, GetTopicsUsersResponse, get_topics_response};
    use bzd_users_api::{GetSourceResponse, GetUserResponse};
    use serde::Serialize;

    use crate::app::error::AppError;

    pub type Response = Source;

    #[derive(Serialize)]
    pub struct Source {
        pub source_id: String,
        pub user: User,
        pub topics: Vec<Topic>,
    }

    #[derive(Serialize)]
    pub struct Topic {
        pub topic_id: String,
    }

    #[derive(Serialize)]
    pub struct User {
        pub user_id: String,
        pub name: String,
        pub abbr: String,
        pub color: String,
    }

    type Qqqq = (
        GetSourceResponse,
        GetUserResponse,
        GetTopicsResponse,
        GetTopicsUsersResponse,
    );

    type Topics = HashMap<String, get_topics_response::Topic>;

    impl TryFrom<Qqqq> for Response {
        type Error = AppError;

        fn try_from(
            (get_source_respose, get_user_response,
            get_topics_response,
            get_topics_users_response
        ): Qqqq,
        ) -> Result<Self, Self::Error> {
            let source = get_source_respose.source.ok_or(AppError::Internal)?;
            let user = get_user_response.user.ok_or(AppError::Internal)?;

            let topics: Topics = get_topics_response
                .topics
                .into_iter()
                .map(|it| (it.topic_id().into(), it))
                .collect();

            Ok(Self {
                source_id: source.source_id().into(),

                user: User {
                    user_id: user.user_id().into(),
                    name: user.name().into(),
                    abbr: user.abbr().into(),
                    color: user.color().into(),
                },

                topics: get_topics_users_response
                    .topics_users
                    .iter()
                    .map(|topic_user| {
                        topics
                            .get(topic_user.topic_id())
                            .map(|topic| Topic {
                                topic_id: topic.topic_id().into(),
                            })
                            .ok_or(AppError::Internal)
                    })
                    .collect::<Result<Vec<Topic>, AppError>>()?,
            })
        }
    }
}
