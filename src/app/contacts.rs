use axum::{Router, extract::State, routing::post};
use bzd_users_api::CreateContactsRequest;

use crate::app::{error::AppError, json::AppJson, state::AppState, user::AppUser};

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(create_contacts))
}

async fn create_contacts(
    State(AppState {
        contacts_service_client,
        ..
    }): State<AppState>,
    user: AppUser,
    AppJson(req): AppJson<create_contacts::Request>,
) -> Result<AppJson<create_contacts::Response>, AppError> {
    let mut req: CreateContactsRequest = req.into();
    req.user_id = Some(user.user_id);

    let res = contacts_service_client
        .clone()
        .create_contacts(req)
        .await?
        .into_inner();

    Ok(AppJson(res.into()))
}

mod create_contacts {
    use bzd_users_api::{CreateContactsRequest, CreateContactsResponse, create_contacts_request};
    use serde::{Deserialize, Serialize};

    #[derive(Deserialize)]
    pub struct Request {
        pub contacts: Vec<Contact>,
    }

    #[derive(Deserialize)]
    pub struct Contact {
        pub phone_number: String,
        pub name: String,
        pub device_contact_id: String,
    }

    impl From<Request> for CreateContactsRequest {
        fn from(req: Request) -> Self {
            Self {
                user_id: None,
                contacts: req.contacts.into_iter().map(Into::into).collect(),
            }
        }
    }

    impl From<Contact> for create_contacts_request::Contact {
        fn from(contact: Contact) -> Self {
            Self {
                phone_number: Some(contact.phone_number),
                name: Some(contact.name),
                device_contact_id: Some(contact.device_contact_id),
            }
        }
    }

    #[derive(Serialize)]
    pub struct Response {}

    impl From<CreateContactsResponse> for Response {
        fn from(_: CreateContactsResponse) -> Self {
            Self {}
        }
    }
}
