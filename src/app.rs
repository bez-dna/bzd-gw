use axum::{Router, routing::get};
use bzd_lib::{error::Error, settings::Settings as _};
use tracing::info;

use crate::app::{settings::AppSettings, state::AppState};

mod auth;
mod error;
mod json;
mod messages;
mod settings;
mod state;
mod topics;
mod user;

pub async fn run() -> Result<(), Error> {
    let settings = AppSettings::new()?;
    let state = AppState::new(settings).await?;

    http(&state).await?;

    Ok(())
}

async fn http(state: &AppState) -> Result<(), Error> {
    let router = Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/healthz", get(|| async {}))
                .nest("/auth", auth::router())
                .nest("/topics", topics::router())
                .nest("/messages", messages::router()),
        )
        .with_state(state.to_owned());

    let listener = tokio::net::TcpListener::bind(&state.settings.http.endpoint).await?;

    info!("app: started on {}", listener.local_addr()?);
    axum::serve(listener, router).await?;

    Ok(())
}
