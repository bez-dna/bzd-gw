use bzd_lib::error::Error;
use bzd_users_api::auth_service_client::AuthServiceClient;
use tonic::transport::Channel;

use crate::app::settings::AppSettings;

#[derive(Clone)]
pub struct AppState {
    pub settings: AppSettings,
    pub auth_service_client: AuthServiceClient<Channel>,
}

impl AppState {
    pub async fn new(settings: AppSettings) -> Result<Self, Error> {
        let auth_service_client =
            Self::auth_service_client(settings.clients.bzd_users.endpoint.clone()).await?;

        Ok(Self {
            settings,
            auth_service_client,
        })
    }

    async fn auth_service_client(dst: String) -> Result<AuthServiceClient<Channel>, Error> {
        let ch = tonic::transport::Endpoint::new(dst)?.connect_lazy();

        Ok(AuthServiceClient::new(ch))
    }
}
