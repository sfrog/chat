use axum::{extract::State, response::IntoResponse, Extension, Json};

use crate::{models::Workspace, AppError, AppState, User};

pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id;
    let users = Workspace::fetch_all_chat_users(ws_id as _, &state.pool).await?;

    Ok(Json(users))
}
