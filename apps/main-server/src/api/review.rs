//! ReviewAssistService and ReviewCommentService handlers.

use axum::{Json, extract::State};
use entities::{ReviewAssistItemStatus, ReviewInlineComment, ReviewInlineCommentStatus};
use rpc_protocol::{requests::*, responses::*};

use crate::{
    error::{AppError, AppResult},
    state::SharedState,
};

// ============================================================================
// ReviewAssistService
// ============================================================================

pub async fn list_assist_items(
    State(state): State<SharedState>,
    Json(req): Json<ListReviewAssistItemsRequest>,
) -> AppResult<Json<ListReviewAssistItemsResponse>> {
    let all_items = state
        .store
        .list_review_assist_items(Some(req.unit_task_id))
        .await?;

    let items = if let Some(status_filter) = req.status {
        all_items
            .into_iter()
            .filter(|item| item.status == status_filter)
            .collect()
    } else {
        all_items
    };

    Ok(Json(ListReviewAssistItemsResponse { items }))
}

pub async fn acknowledge_assist_item(
    State(state): State<SharedState>,
    Json(req): Json<AcknowledgeReviewAssistItemRequest>,
) -> AppResult<Json<AcknowledgeReviewAssistItemResponse>> {
    let mut item = state
        .store
        .get_review_assist_item(req.item_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Review assist item {} not found", req.item_id))
        })?;

    item.status = ReviewAssistItemStatus::Acknowledged;
    item.updated_at = chrono::Utc::now();
    state.store.update_review_assist_item(item).await?;

    Ok(Json(AcknowledgeReviewAssistItemResponse {}))
}

pub async fn dismiss_assist_item(
    State(state): State<SharedState>,
    Json(req): Json<DismissReviewAssistItemRequest>,
) -> AppResult<Json<DismissReviewAssistItemResponse>> {
    let mut item = state
        .store
        .get_review_assist_item(req.item_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Review assist item {} not found", req.item_id))
        })?;

    item.status = ReviewAssistItemStatus::Dismissed;
    item.updated_at = chrono::Utc::now();
    state.store.update_review_assist_item(item).await?;

    Ok(Json(DismissReviewAssistItemResponse {}))
}

// ============================================================================
// ReviewCommentService
// ============================================================================

pub async fn list_inline_comments(
    State(state): State<SharedState>,
    Json(req): Json<ListReviewInlineCommentsRequest>,
) -> AppResult<Json<ListReviewInlineCommentsResponse>> {
    let all_comments = state
        .store
        .list_review_inline_comments(req.unit_task_id)
        .await?;

    let comments = if let Some(status_filter) = req.status {
        all_comments
            .into_iter()
            .filter(|c| c.status == status_filter)
            .collect()
    } else {
        all_comments
    };

    Ok(Json(ListReviewInlineCommentsResponse { comments }))
}

pub async fn create_inline_comment(
    State(state): State<SharedState>,
    Json(req): Json<CreateReviewInlineCommentRequest>,
) -> AppResult<Json<CreateReviewInlineCommentResponse>> {
    let mut comment = ReviewInlineComment::new(
        req.unit_task_id,
        req.file_path,
        req.side,
        req.line_number,
        req.body,
    );
    if let Some(sub_task_id) = req.sub_task_id {
        comment.sub_task_id = Some(sub_task_id);
    }

    let comment = state.store.create_review_inline_comment(comment).await?;
    Ok(Json(CreateReviewInlineCommentResponse { comment }))
}

pub async fn resolve_inline_comment(
    State(state): State<SharedState>,
    Json(req): Json<ResolveReviewInlineCommentRequest>,
) -> AppResult<Json<ResolveReviewInlineCommentResponse>> {
    let mut comment = state
        .store
        .get_review_inline_comment(req.comment_id)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Review comment {} not found", req.comment_id))
        })?;

    comment.status = ReviewInlineCommentStatus::Resolved;
    comment.updated_at = chrono::Utc::now();
    state.store.update_review_inline_comment(comment).await?;

    Ok(Json(ResolveReviewInlineCommentResponse {}))
}
