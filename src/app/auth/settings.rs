use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct AuthSettings {
    pub public_key_file: String,
}
