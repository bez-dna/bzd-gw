use std::collections::HashSet;

use axum::{
    Router,
    extract::State,
    routing::{get, post},
};
use bzd_users_api::{CreateSourceRequest, GetSourcesRequest, GetUsersRequest};

use crate::app::{error::AppError, json::AppJson, state::AppState, user::AppUser};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_sources))
        .route("/", post(create_source))
}

async fn get_sources(
    State(AppState {
        sources_service_client,
        users_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
) -> Result<AppJson<get_sources::Response>, AppError> {
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

mod get_sources {
    use std::collections::HashMap;

    use bzd_users_api::{
        GetSourcesResponse, GetUsersResponse, get_sources_response, get_users_response,
    };
    use serde::Serialize;

    use crate::app::error::AppError;

    #[derive(Serialize)]
    pub struct Response {
        pub sources: Vec<Source>,
        pub contacts: Vec<Contact>,
    }

    #[derive(Serialize)]
    pub struct Source {
        pub source_id: String,
        pub user_id: String,
        pub name: String,
        pub phone: String,
        pub abbr: String,
        pub color: String,
    }

    #[derive(Serialize)]
    pub struct Contact {
        pub contact_id: String,
        pub contact_name: String,
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

            let contacts = get_sources_response
                .contacts
                .into_iter()
                .map(|contact| (contact, &users).try_into())
                .collect::<Result<Vec<Contact>, _>>()?;

            let sources = get_sources_response
                .sources
                .into_iter()
                .map(|source| (source, &users).try_into())
                .collect::<Result<Vec<Source>, _>>()?;

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
                user_id: user.user_id().into(),
                name: user.name().into(),
                phone: user.phone().into(),
                abbr: user.abbr().into(),
                color: user.color().into(),
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
                user_id: user.user_id().into(),
                name: user.name().into(),
                phone: user.phone().into(),
                abbr: user.abbr().into(),
                color: user.color().into(),
            })
        }
    }
}

async fn create_source(
    State(AppState {
        sources_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
    AppJson(req): AppJson<create_source::Request>,
) -> Result<AppJson<create_source::Response>, AppError> {
    let res = sources_service_client
        .clone()
        .create_source(CreateSourceRequest {
            user_id: Some(user.user_id.into()),
            source_user_id: Some(req.user_id.into()),
        })
        .await?
        .into_inner();

    Ok(AppJson(res.into()))
}

mod create_source {
    use bzd_users_api::CreateSourceResponse;
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct Request {
        pub user_id: String,
    }

    #[derive(Serialize)]
    pub struct Response {
        pub source_id: String,
    }

    impl From<CreateSourceResponse> for Response {
        fn from(res: CreateSourceResponse) -> Self {
            Self {
                source_id: res.source_id().into(),
            }
        }
    }
}
