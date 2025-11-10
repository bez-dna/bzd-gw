use std::collections::HashSet;

use axum::{Router, extract::State, routing::get};
use bzd_messages_api::{GetTopicsRequest, GetTopicsUsersRequest};
use bzd_users_api::{GetSourcesRequest, GetUsersRequest};

use crate::app::{error::AppError, json::AppJson, state::AppState, user::AppUser};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_users))
}

async fn get_users(
    State(AppState {
        sources_service_client,
        users_service_client,
        topics_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
) -> Result<AppJson<get_users::Response>, AppError> {
    let req = GetSourcesRequest {
        user_id: Some(user.user_id.clone().into()),
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
            user_ids: user_ids.clone().into_iter().collect(),
        })
        .await?
        .into_inner();

    let get_topics_response = topics_service_client
        .clone()
        .get_topics(GetTopicsRequest {
            user_ids: user_ids.into_iter().collect(),
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
            user_id: Some(user.user_id.into()),
        })
        .await?
        .into_inner();

    Ok(AppJson(
        (
            get_sources_response,
            get_users_response,
            get_topics_response,
            get_topics_users_response,
        )
            .try_into()?,
    ))
}

mod get_users {
    use std::collections::HashMap;

    use bzd_messages_api::{
        GetTopicsResponse, GetTopicsUsersResponse, get_topics_response, get_topics_users_response,
    };
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
    type Topics = Vec<get_topics_response::Topic>;
    type TopicsUsers = HashMap<String, get_topics_users_response::TopicUser>;

    // TODO: нужно придумать имя для такой темп структуры
    type Responses = (
        GetSourcesResponse,
        GetUsersResponse,
        GetTopicsResponse,
        GetTopicsUsersResponse,
    );

    impl TryFrom<Responses> for Response {
        type Error = AppError;

        fn try_from(
            (
                get_sources_response,
                get_users_response,
                get_topics_response,
                get_topics_users_response,
            ): Responses,
        ) -> Result<Self, Self::Error> {
            let users: Users = get_users_response
                .users
                .into_iter()
                .map(|it| (it.user_id().into(), it))
                .collect();

            let topics: Topics = get_topics_response.topics;

            let topics_users: TopicsUsers = get_topics_users_response
                .topics_users
                .into_iter()
                .map(|it| (it.topic_id().into(), it))
                .collect();

            let contacts: Contacts = get_sources_response
                .contacts
                .into_iter()
                .map(|contact| (contact, &users).try_into())
                .collect::<Result<_, _>>()?;

            let sources: Sources = get_sources_response
                .sources
                .into_iter()
                .map(|source| (source, &users, &topics, &topics_users).try_into())
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

    impl TryFrom<(get_sources_response::Source, &Users, &Topics, &TopicsUsers)> for Source {
        type Error = AppError;

        fn try_from(
            (source, users, topics, topics_users): (
                get_sources_response::Source,
                &Users,
                &Topics,
                &TopicsUsers,
            ),
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

                topics: topics
                    .iter()
                    .filter(|it| it.user_id() == source.source_user_id())
                    .map(|topic| Topic {
                        topic_id: topic.topic_id().into(),
                        title: topic.title().into(),
                        topic_user: topics_users.get(topic.topic_id()).map(|it| TopicUser {
                            topic_user_id: it.topic_user_id().into(),
                        }),
                    })
                    .collect(),
            })
        }
    }
}
