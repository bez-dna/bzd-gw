use axum::{
    RequestPartsExt as _,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use jsonwebtoken::{Algorithm, TokenData, Validation, decode};
use serde::Deserialize;

use crate::app::{error::AppError, state::AppState};

impl<S> FromRequestParts<S> for CurrentUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let AppState { decoding_key, .. } = AppState::from_ref(state);

        let bearer = parts
            .extract::<Option<TypedHeader<Authorization<Bearer>>>>()
            .await?
            .map(|TypedHeader(Authorization(bearer))| bearer);

        let user_id = match bearer {
            Some(bearer) => {
                let TokenData { claims, .. } = decode::<Claims>(
                    bearer.token(),
                    &decoding_key,
                    &Validation::new(Algorithm::RS256),
                )?;

                Some(claims.sub)
            }
            None => None,
        };

        let user = CurrentUser { user_id };

        Ok(user)
    }
}

#[derive(Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    // pub exp: usize,
}

#[derive(Deserialize, Debug)]
pub struct CurrentUser {
    pub user_id: Option<String>,
}
