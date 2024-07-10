use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{AppError, AppState, User};

#[derive(Debug, Deserialize, Serialize)]
struct AuthOutput {
    token: String,
}

pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<crate::models::SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::verify(&input, &state.pool).await?;

    match user {
        Some(user) => {
            let token = state.ek.sign(user)?;
            Ok((StatusCode::OK, Json(AuthOutput { token })))
        }
        None => Err(AppError::Unauthorized),
    }
}

pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<crate::models::CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::create(&input, &state.pool).await?;
    let token = state.ek.sign(user)?;
    Ok((StatusCode::CREATED, Json(AuthOutput { token })))
}

#[cfg(test)]
mod tests {
    use http_body_util::BodyExt as _;

    use crate::{
        models::{CreateUser, SigninUser},
        AppConfig,
    };

    use super::*;

    #[tokio::test]
    async fn test_signup_handler() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::try_new_for_test(AppConfig::load()?).await?;
        let input = CreateUser::new("default", "fullname", "email", "password");
        let ret = signup_handler(State(state), Json(input))
            .await?
            .into_response();

        assert_eq!(ret.status(), StatusCode::CREATED);

        let body = ret.into_body().collect().await?.to_bytes();
        let output = serde_json::from_slice::<AuthOutput>(&body)?;
        assert_ne!(output.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn test_signin_handler() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::try_new_for_test(AppConfig::load()?).await?;
        let user = CreateUser::new("default", "fullname", "email", "password");
        User::create(&user, &state.pool).await?;

        let input = SigninUser::new("email", "password");
        let ret = signin_handler(State(state), Json(input))
            .await?
            .into_response();

        assert_eq!(ret.status(), StatusCode::OK);
        let body = ret.into_body().collect().await?.to_bytes();
        let output = serde_json::from_slice::<AuthOutput>(&body)?;
        assert_ne!(output.token, "");

        Ok(())
    }
}
