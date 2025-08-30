use bzd_lib::settings::Settings;

use bzd_lib::settings::HttpSettings;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct AppSettings {
    pub http: HttpSettings,
}

impl Settings<AppSettings> for AppSettings {}
