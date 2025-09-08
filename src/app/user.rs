use axum_extra::headers::authorization::Bearer;
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation, decode};
use serde::Deserialize;

use crate::app::error::AppError;

fn jwt_2_user(bearer: Bearer, public_key: &Vec<u8>) -> Result<AppUser, AppError> {
    let TokenData { claims, .. } = decode::<Claims>(
        bearer.token(),
        &DecodingKey::from_rsa_pem(public_key)?,
        &Validation::new(Algorithm::RS256),
    )?;

    Ok(AppUser {
        user_id: claims.sub,
    })
}

mod jwt_2_user {
    use axum::{
        RequestPartsExt as _,
        extract::{FromRef, FromRequestParts, OptionalFromRequestParts},
        http::request::Parts,
    };
    use axum_extra::{
        TypedHeader,
        headers::{Authorization, authorization::Bearer},
    };

    use crate::app::{
        error::AppError,
        state::AppState,
        user::{AppUser, jwt_2_user},
    };

    impl<S> FromRequestParts<S> for AppUser
    where
        AppState: FromRef<S>,
        S: Send + Sync,
    {
        type Rejection = AppError;

        async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
            let AppState { public_key, .. } = AppState::from_ref(state);

            let TypedHeader(Authorization(bearer)) = parts
                .extract::<TypedHeader<Authorization<Bearer>>>()
                .await?;

            let user = jwt_2_user(bearer, &public_key)?;

            Ok(user)
        }
    }

    impl<S> OptionalFromRequestParts<S> for AppUser
    where
        AppState: FromRef<S>,
        S: Send + Sync,
    {
        type Rejection = AppError;

        async fn from_request_parts(
            parts: &mut Parts,
            state: &S,
        ) -> Result<Option<Self>, Self::Rejection> {
            match <AppUser as FromRequestParts<S>>::from_request_parts(parts, state).await {
                Ok(user) => Ok(Some(user)),
                Err(_) => Ok(None),
            }
        }
    }
}

#[derive(Deserialize)]
pub struct Claims {
    pub sub: String,
    // pub exp: usize,
}

#[derive(Deserialize, Debug)]
pub struct AppUser {
    pub user_id: String,
}
