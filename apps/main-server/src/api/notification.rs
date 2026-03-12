//! NotificationService handlers.

use axum::{Json, extract::State};
use rpc_protocol::{requests::*, responses::*};

use crate::{error::AppResult, state::SharedState};

pub async fn list(
    State(state): State<SharedState>,
    Json(req): Json<ListNotificationsRequest>,
) -> AppResult<Json<ListNotificationsResponse>> {
    let all_notifications = state
        .store
        .list_notifications(Some(req.workspace_id), req.unread_only)
        .await?;

    let total_count = all_notifications.len() as i32;
    let unread_count = all_notifications.iter().filter(|n| !n.is_read()).count() as i32;

    let limit = if req.limit > 0 {
        req.limit as usize
    } else {
        usize::MAX
    };
    let offset = if req.offset > 0 {
        req.offset as usize
    } else {
        0
    };

    let notifications = all_notifications
        .into_iter()
        .skip(offset)
        .take(limit)
        .collect();

    Ok(Json(ListNotificationsResponse {
        notifications,
        total_count,
        unread_count,
    }))
}

pub async fn mark_read(
    State(state): State<SharedState>,
    Json(req): Json<MarkNotificationReadRequest>,
) -> AppResult<Json<MarkNotificationReadResponse>> {
    state
        .store
        .mark_notification_read(req.notification_id)
        .await?;
    Ok(Json(MarkNotificationReadResponse {}))
}

pub async fn mark_all_read(
    State(state): State<SharedState>,
    Json(req): Json<MarkAllNotificationsReadRequest>,
) -> AppResult<Json<MarkAllNotificationsReadResponse>> {
    // Get all unread notifications first to count them.
    let unread = state
        .store
        .list_notifications(Some(req.workspace_id), true)
        .await?;
    let marked_count = unread.len() as i32;

    state
        .store
        .mark_all_notifications_read(req.workspace_id)
        .await?;

    Ok(Json(MarkAllNotificationsReadResponse { marked_count }))
}
