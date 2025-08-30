use bzd_lib::error::Error;

use crate::app::settings::AppSettings;

#[derive(Clone)]
pub struct AppState {
    pub settings: AppSettings,
}

impl AppState {
    pub async fn new(settings: AppSettings) -> Result<Self, Error> {
        Ok(Self { settings })
    }
}
