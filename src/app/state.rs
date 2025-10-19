use std::sync::Arc;

use bzd_lib::error::Error;
use bzd_messages_api::{
    messages_service_client::MessagesServiceClient, topics_service_client::TopicsServiceClient,
};
use bzd_users_api::{
    auth_service_client::AuthServiceClient, contacts_service_client::ContactsServiceClient,
    sources_service_client::SourcesServiceClient, users_service_client::UsersServiceClient,
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
    pub sources_service_client: SourcesServiceClient<Channel>,
    pub decoding_key: Arc<DecodingKey>,
}

impl AppState {
    pub async fn new(settings: AppSettings) -> Result<Self, Error> {
        let auth_service_client = Self::create_service_client(
            settings.clients.bzd_users.endpoint.clone(),
            AuthServiceClient::new,
        )
        .await?;

        let users_service_client = Self::create_service_client(
            settings.clients.bzd_users.endpoint.clone(),
            UsersServiceClient::new,
        )
        .await?;

        let contacts_service_client = Self::create_service_client(
            settings.clients.bzd_users.endpoint.clone(),
            ContactsServiceClient::new,
        )
        .await?;

        let sources_service_client = Self::create_service_client(
            settings.clients.bzd_users.endpoint.clone(),
            SourcesServiceClient::new,
        )
        .await?;

        let messages_service_client = Self::create_service_client(
            settings.clients.bzd_messages.endpoint.clone(),
            MessagesServiceClient::new,
        )
        .await?;

        let topics_service_client = Self::create_service_client(
            settings.clients.bzd_messages.endpoint.clone(),
            TopicsServiceClient::new,
        )
        .await?;

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
            sources_service_client,
            messages_service_client,
            topics_service_client,
            decoding_key,
        })
    }

    async fn create_service_client<T, F>(dst: String, ctor: F) -> Result<T, AppError>
    where
        F: FnOnce(Channel) -> T,
    {
        let ch = tonic::transport::Endpoint::new(dst)?.connect_lazy();

        Ok(ctor(ch))
    }
}
