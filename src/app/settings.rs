use bzd_lib::settings::Settings;

use bzd_lib::settings::HttpSettings;
use serde::Deserialize;

use crate::app::auth::settings::AuthSettings;

#[derive(Deserialize, Clone)]
pub struct AppSettings {
    pub http: HttpSettings,
    pub auth: AuthSettings,
    pub clients: ClientsSettings,
}

#[derive(Deserialize, Clone)]
pub struct ClientsSettings {
    pub bzd_users: ClientSettings,
    pub bzd_messages: ClientSettings,
}

#[derive(Deserialize, Clone)]
pub struct ClientSettings {
    pub endpoint: String,
}

impl Settings<AppSettings> for AppSettings {}
