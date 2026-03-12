//! BadgeThemeService handlers.

use axum::{Json, extract::State};
use entities::BadgeTheme;
use rpc_protocol::{requests::*, responses::*};

use crate::{error::AppResult, state::SharedState};

pub async fn list(
    State(state): State<SharedState>,
    Json(req): Json<ListBadgeThemesRequest>,
) -> AppResult<Json<ListBadgeThemesResponse>> {
    let themes = state.store.list_badge_themes(req.workspace_id).await?;
    Ok(Json(ListBadgeThemesResponse { themes }))
}

pub async fn upsert(
    State(state): State<SharedState>,
    Json(req): Json<UpsertBadgeThemeRequest>,
) -> AppResult<Json<UpsertBadgeThemeResponse>> {
    let theme = BadgeTheme::new(req.workspace_id, req.action_type, req.color_key);
    let theme = state.store.upsert_badge_theme(theme).await?;
    Ok(Json(UpsertBadgeThemeResponse { theme }))
}
