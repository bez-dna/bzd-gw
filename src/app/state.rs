use std::sync::Arc;

use bzd_lib::error::Error;
use bzd_messages_api::{
    messages_service_client::MessagesServiceClient, topics_service_client::TopicsServiceClient,
};
use bzd_users_api::{
    auth_service_client::AuthServiceClient, contacts_service_client::ContactsServiceClient,
    users_service_client::UsersServiceClient,
};
use jsonwebtoken::DecodingKey;
use tokio::fs;
use tonic::transport::Channel;

use crate::app::{error::AppError, settings::AppSettings};

#[derive(Clone)]
pub struct AppState {
    pub settings: AppSettings,
    pub auth_service_client: AuthServiceClient<Channel>,
    pub users_service_client: UsersServiceClient<Channel>,
    pub contacts_service_client: ContactsServiceClient<Channel>,
    pub messages_service_client: MessagesServiceClient<Channel>,
    pub topics_service_client: TopicsServiceClient<Channel>,
    pub decoding_key: Arc<DecodingKey>,
}

impl AppState {
    pub async fn new(settings: AppSettings) -> Result<Self, Error> {
        let auth_service_client =
            Self::auth_service_client(settings.clients.bzd_users.endpoint.clone()).await?;

        let users_service_client =
            Self::users_service_client(settings.clients.bzd_users.endpoint.clone()).await?;

        let contacts_service_client =
            Self::contacts_service_client(settings.clients.bzd_users.endpoint.clone()).await?;

        let messages_service_client =
            Self::messages_service_client(settings.clients.bzd_messages.endpoint.clone()).await?;

        let topics_service_client =
            Self::topics_service_client(settings.clients.bzd_messages.endpoint.clone()).await?;

        let public_key = fs::read_to_string(&settings.auth.public_key_file)
            .await?
            .into_bytes();

        let decoding_key =
            Arc::new(DecodingKey::from_rsa_pem(&public_key).map_err(|_| AppError::Internal)?);

        Ok(Self {
            settings,
            auth_service_client,
            users_service_client,
            contacts_service_client,
            messages_service_client,
            topics_service_client,
            decoding_key,
        })
    }

    async fn auth_service_client(dst: String) -> Result<AuthServiceClient<Channel>, Error> {
        let ch = tonic::transport::Endpoint::new(dst)?.connect_lazy();

        Ok(AuthServiceClient::new(ch))
    }

    async fn users_service_client(dst: String) -> Result<UsersServiceClient<Channel>, Error> {
        let ch = tonic::transport::Endpoint::new(dst)?.connect_lazy();

        Ok(UsersServiceClient::new(ch))
    }

    async fn contacts_service_client(dst: String) -> Result<ContactsServiceClient<Channel>, Error> {
        let ch = tonic::transport::Endpoint::new(dst)?.connect_lazy();

        Ok(ContactsServiceClient::new(ch))
    }

    async fn messages_service_client(dst: String) -> Result<MessagesServiceClient<Channel>, Error> {
        let ch = tonic::transport::Endpoint::new(dst)?.connect_lazy();

        Ok(MessagesServiceClient::new(ch))
    }

    async fn topics_service_client(dst: String) -> Result<TopicsServiceClient<Channel>, Error> {
        let ch = tonic::transport::Endpoint::new(dst)?.connect_lazy();

        Ok(TopicsServiceClient::new(ch))
    }
}
