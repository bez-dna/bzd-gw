use bzd_lib::settings::Settings;

use bzd_lib::settings::HttpSettings;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct AppSettings {
    pub http: HttpSettings,
    pub clients: ClientsSettings,
}

#[derive(Deserialize, Clone)]
pub struct ClientsSettings {
    pub bzd_users: ClientSettings,
}

#[derive(Deserialize, Clone)]
pub struct ClientSettings {
    pub endpoint: String,
}

impl Settings<AppSettings> for AppSettings {}
