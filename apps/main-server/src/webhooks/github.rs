//! GitHub webhook handler implementation.

use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use std::sync::Arc;
use task_store::TaskStore;
use tracing::{error, info, warn};

use crate::state::AppState;

use super::{
    CheckRunConclusion, CheckRunPayload, CheckRunStatus, GitHubEventType,
    PullRequestReviewCommentPayload, PullRequestReviewPayload, ReviewAction, ReviewState,
    WebhookResult,
};

/// Creates the webhook router.
pub fn webhook_router<S: TaskStore + 'static>() -> Router<Arc<AppState<S>>> {
    Router::new().route("/github", post(handle_github_webhook::<S>))
}

/// Extracts the webhook signature from headers.
fn get_signature(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("X-Hub-Signature-256")
        .or_else(|| headers.get("X-Hub-Signature"))
        .and_then(|v| v.to_str().ok())
}

/// Extracts the event type from headers.
fn get_event_type(headers: &HeaderMap) -> GitHubEventType {
    headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .map(GitHubEventType::from_header)
        .unwrap_or(GitHubEventType::Unknown)
}

/// Extracts the delivery ID from headers.
fn get_delivery_id(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("X-GitHub-Delivery")
        .and_then(|v| v.to_str().ok())
}

/// Verifies the webhook signature using HMAC-SHA256.
///
/// Note: In production, you should verify the signature against your webhook secret.
/// This is a placeholder that always returns true for development.
fn verify_signature(_secret: &str, _signature: &str, _payload: &[u8]) -> bool {
    // TODO: Implement proper HMAC-SHA256 verification
    // For now, we accept all webhooks. In production:
    // 1. Extract the hash from the signature (sha256=XXXX)
    // 2. Compute HMAC-SHA256 of the payload using the secret
    // 3. Compare using constant-time comparison
    true
}

/// Main webhook handler.
async fn handle_github_webhook<S: TaskStore>(
    State(state): State<Arc<AppState<S>>>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let event_type = get_event_type(&headers);
    let delivery_id = get_delivery_id(&headers).unwrap_or("unknown");

    info!(
        delivery_id = %delivery_id,
        event_type = ?event_type,
        "Received GitHub webhook"
    );

    // Verify signature if webhook secret is configured
    if let Some(secret) = &state.config.webhook_secret {
        if let Some(signature) = get_signature(&headers) {
            if !verify_signature(secret, signature, &body) {
                warn!(delivery_id = %delivery_id, "Invalid webhook signature");
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(WebhookResult::error("Invalid signature")),
                );
            }
        } else {
            warn!(delivery_id = %delivery_id, "Missing webhook signature");
            return (
                StatusCode::UNAUTHORIZED,
                Json(WebhookResult::error("Missing signature")),
            );
        }
    }

    // Process based on event type
    let result = match event_type {
        GitHubEventType::PullRequestReview => {
            handle_pull_request_review(&state, &body, delivery_id).await
        }
        GitHubEventType::PullRequestReviewComment => {
            handle_review_comment(&state, &body, delivery_id).await
        }
        GitHubEventType::CheckRun => handle_check_run(&state, &body, delivery_id).await,
        GitHubEventType::CheckSuite => {
            WebhookResult::skipped("Check suite events not processed")
        }
        GitHubEventType::Status => WebhookResult::skipped("Status events not processed"),
        _ => WebhookResult::skipped(format!("Event type {:?} not handled", event_type)),
    };

    let status = if result.success {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    (status, Json(result))
}

/// Handles pull request review events.
async fn handle_pull_request_review<S: TaskStore>(
    state: &Arc<AppState<S>>,
    body: &[u8],
    delivery_id: &str,
) -> WebhookResult {
    let payload: PullRequestReviewPayload = match serde_json::from_slice(body) {
        Ok(p) => p,
        Err(e) => {
            error!(delivery_id = %delivery_id, error = %e, "Failed to parse review payload");
            return WebhookResult::error(format!("Failed to parse payload: {}", e));
        }
    };

    info!(
        delivery_id = %delivery_id,
        action = ?payload.action,
        review_state = ?payload.review.state,
        pr_number = %payload.pull_request.number,
        repo = %payload.repository.full_name,
        "Processing pull request review"
    );

    // Only process submitted reviews
    if payload.action != ReviewAction::Submitted {
        return WebhookResult::skipped("Review not submitted");
    }

    // Only process reviews with changes requested or comments
    match payload.review.state {
        ReviewState::ChangesRequested | ReviewState::Commented => {
            // Check if auto-fix is enabled and create task
            if let Some(body) = &payload.review.body {
                if !body.trim().is_empty() {
                    return create_auto_fix_task_for_review(
                        state,
                        &payload,
                        body,
                        delivery_id,
                    )
                    .await;
                }
            }
            WebhookResult::skipped("No review body to process")
        }
        _ => WebhookResult::skipped(format!("Review state {:?} not actionable", payload.review.state)),
    }
}

/// Handles pull request review comment events.
async fn handle_review_comment<S: TaskStore>(
    state: &Arc<AppState<S>>,
    body: &[u8],
    delivery_id: &str,
) -> WebhookResult {
    let payload: PullRequestReviewCommentPayload = match serde_json::from_slice(body) {
        Ok(p) => p,
        Err(e) => {
            error!(delivery_id = %delivery_id, error = %e, "Failed to parse comment payload");
            return WebhookResult::error(format!("Failed to parse payload: {}", e));
        }
    };

    info!(
        delivery_id = %delivery_id,
        action = %payload.action,
        pr_number = %payload.pull_request.number,
        repo = %payload.repository.full_name,
        comment_path = %payload.comment.path,
        "Processing review comment"
    );

    // Only process created comments
    if payload.action != "created" {
        return WebhookResult::skipped("Comment not created");
    }

    // Create auto-fix task for the comment
    create_auto_fix_task_for_comment(state, &payload, delivery_id).await
}

/// Handles check run events.
async fn handle_check_run<S: TaskStore>(
    state: &Arc<AppState<S>>,
    body: &[u8],
    delivery_id: &str,
) -> WebhookResult {
    let payload: CheckRunPayload = match serde_json::from_slice(body) {
        Ok(p) => p,
        Err(e) => {
            error!(delivery_id = %delivery_id, error = %e, "Failed to parse check run payload");
            return WebhookResult::error(format!("Failed to parse payload: {}", e));
        }
    };

    info!(
        delivery_id = %delivery_id,
        action = %payload.action,
        check_name = %payload.check_run.name,
        status = ?payload.check_run.status,
        conclusion = ?payload.check_run.conclusion,
        repo = %payload.repository.full_name,
        "Processing check run"
    );

    // Only process completed check runs
    if payload.check_run.status != CheckRunStatus::Completed {
        return WebhookResult::skipped("Check run not completed");
    }

    // Only process failed check runs
    match payload.check_run.conclusion {
        Some(CheckRunConclusion::Failure) | Some(CheckRunConclusion::TimedOut) => {
            create_auto_fix_task_for_ci_failure(state, &payload, delivery_id).await
        }
        _ => WebhookResult::skipped(format!(
            "Check run conclusion {:?} not actionable",
            payload.check_run.conclusion
        )),
    }
}

/// Creates an auto-fix task for a review.
async fn create_auto_fix_task_for_review<S: TaskStore>(
    _state: &Arc<AppState<S>>,
    payload: &PullRequestReviewPayload,
    _body: &str,
    delivery_id: &str,
) -> WebhookResult {
    // TODO: Look up the UnitTask associated with this PR
    // TODO: Check if auto-fix is enabled for this repository
    // TODO: Check max auto-fix attempts
    // TODO: Create AgentTask with review feedback

    info!(
        delivery_id = %delivery_id,
        pr_number = %payload.pull_request.number,
        reviewer = %payload.review.user.login,
        "Would create auto-fix task for review"
    );

    // For now, just acknowledge the webhook
    WebhookResult::success(format!(
        "Review from {} on PR #{} acknowledged. Auto-fix task creation pending implementation.",
        payload.review.user.login, payload.pull_request.number
    ))
}

/// Creates an auto-fix task for a review comment.
async fn create_auto_fix_task_for_comment<S: TaskStore>(
    _state: &Arc<AppState<S>>,
    payload: &PullRequestReviewCommentPayload,
    delivery_id: &str,
) -> WebhookResult {
    // TODO: Look up the UnitTask associated with this PR
    // TODO: Check if auto-fix is enabled for this repository
    // TODO: Check reviewer permissions (write_access_only filter)
    // TODO: Check max auto-fix attempts
    // TODO: Create AgentTask with comment feedback

    info!(
        delivery_id = %delivery_id,
        pr_number = %payload.pull_request.number,
        commenter = %payload.comment.user.login,
        file = %payload.comment.path,
        "Would create auto-fix task for comment"
    );

    WebhookResult::success(format!(
        "Comment from {} on {} acknowledged. Auto-fix task creation pending implementation.",
        payload.comment.user.login, payload.comment.path
    ))
}

/// Creates an auto-fix task for a CI failure.
async fn create_auto_fix_task_for_ci_failure<S: TaskStore>(
    _state: &Arc<AppState<S>>,
    payload: &CheckRunPayload,
    delivery_id: &str,
) -> WebhookResult {
    // TODO: Find the PR associated with this check run (via head_sha)
    // TODO: Look up the UnitTask associated with this PR
    // TODO: Check if auto-fix CI failures is enabled
    // TODO: Check max auto-fix attempts
    // TODO: Fetch CI logs if available
    // TODO: Create AgentTask with CI failure context

    info!(
        delivery_id = %delivery_id,
        check_name = %payload.check_run.name,
        head_sha = %payload.check_run.head_sha,
        "Would create auto-fix task for CI failure"
    );

    WebhookResult::success(format!(
        "CI failure in '{}' acknowledged. Auto-fix task creation pending implementation.",
        payload.check_run.name
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_from_header() {
        assert_eq!(
            GitHubEventType::from_header("pull_request"),
            GitHubEventType::PullRequest
        );
        assert_eq!(
            GitHubEventType::from_header("pull_request_review"),
            GitHubEventType::PullRequestReview
        );
        assert_eq!(
            GitHubEventType::from_header("check_run"),
            GitHubEventType::CheckRun
        );
        assert_eq!(
            GitHubEventType::from_header("unknown_event"),
            GitHubEventType::Unknown
        );
    }

    #[test]
    fn test_webhook_result() {
        let success = WebhookResult::success("OK");
        assert!(success.success);
        assert_eq!(success.message, "OK");
        assert!(success.auto_fix_task_id.is_none());

        let task_id = uuid::Uuid::new_v4();
        let with_task = WebhookResult::with_task("Created", task_id);
        assert!(with_task.success);
        assert_eq!(with_task.auto_fix_task_id, Some(task_id));

        let skipped = WebhookResult::skipped("Not needed");
        assert!(skipped.success);

        let error = WebhookResult::error("Failed");
        assert!(!error.success);
    }
}
