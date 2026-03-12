//! API route registration.

pub mod badge;
pub mod events;
pub mod notification;
pub mod pr;
pub mod repository;
pub mod review;
pub mod secrets;
pub mod session;
pub mod subtask;
pub mod task;
pub mod worker;
pub mod workspace;

use axum::{Router, routing::post};

use crate::state::SharedState;

/// Creates and returns the router with all API routes.
pub fn create_router() -> Router<SharedState> {
    Router::new()
        // WorkspaceService
        .route("/WorkspaceService/Create", post(workspace::create))
        .route("/WorkspaceService/List", post(workspace::list))
        .route("/WorkspaceService/Get", post(workspace::get))
        .route("/WorkspaceService/Update", post(workspace::update))
        .route("/WorkspaceService/Delete", post(workspace::delete))
        // RepositoryService
        .route("/RepositoryService/Add", post(repository::add))
        .route("/RepositoryService/List", post(repository::list))
        .route("/RepositoryService/Get", post(repository::get))
        .route("/RepositoryService/Remove", post(repository::remove))
        .route(
            "/RepositoryService/CreateGroup",
            post(repository::create_group),
        )
        .route(
            "/RepositoryService/ListGroups",
            post(repository::list_groups),
        )
        .route(
            "/RepositoryService/UpdateGroup",
            post(repository::update_group),
        )
        .route(
            "/RepositoryService/DeleteGroup",
            post(repository::delete_group),
        )
        // TaskService
        .route("/TaskService/Create", post(task::create))
        .route("/TaskService/List", post(task::list))
        .route("/TaskService/Get", post(task::get))
        .route("/TaskService/Cancel", post(task::cancel))
        .route("/TaskService/Delete", post(task::delete))
        // SubTaskService
        .route("/SubTaskService/List", post(subtask::list))
        .route("/SubTaskService/Get", post(subtask::get))
        .route("/SubTaskService/Approve", post(subtask::approve))
        .route("/SubTaskService/ApprovePlan", post(subtask::approve_plan))
        .route("/SubTaskService/RevisePlan", post(subtask::revise_plan))
        .route("/SubTaskService/Retry", post(subtask::retry))
        // SessionService
        .route("/SessionService/List", post(session::list))
        .route("/SessionService/Get", post(session::get))
        .route("/SessionService/GetOutput", post(session::get_output))
        .route("/SessionService/Stop", post(session::stop))
        // PrManagementService
        .route(
            "/PrManagementService/CreateTracking",
            post(pr::create_tracking),
        )
        .route("/PrManagementService/GetTracking", post(pr::get_tracking))
        .route(
            "/PrManagementService/ListTrackings",
            post(pr::list_trackings),
        )
        .route(
            "/PrManagementService/TriggerAutoFix",
            post(pr::trigger_auto_fix),
        )
        // ReviewAssistService
        .route(
            "/ReviewAssistService/ListItems",
            post(review::list_assist_items),
        )
        .route(
            "/ReviewAssistService/Acknowledge",
            post(review::acknowledge_assist_item),
        )
        .route(
            "/ReviewAssistService/Dismiss",
            post(review::dismiss_assist_item),
        )
        // ReviewCommentService
        .route(
            "/ReviewCommentService/List",
            post(review::list_inline_comments),
        )
        .route(
            "/ReviewCommentService/Create",
            post(review::create_inline_comment),
        )
        .route(
            "/ReviewCommentService/Resolve",
            post(review::resolve_inline_comment),
        )
        // BadgeThemeService
        .route("/BadgeThemeService/List", post(badge::list))
        .route("/BadgeThemeService/Upsert", post(badge::upsert))
        // NotificationService
        .route("/NotificationService/List", post(notification::list))
        .route(
            "/NotificationService/MarkRead",
            post(notification::mark_read),
        )
        .route(
            "/NotificationService/MarkAllRead",
            post(notification::mark_all_read),
        )
        // EventStreamService (SSE)
        .route("/EventStreamService/Subscribe", post(events::subscribe))
        // WorkerService
        .route("/WorkerService/Register", post(worker::register))
        .route("/WorkerService/Heartbeat", post(worker::heartbeat))
        .route("/WorkerService/Unregister", post(worker::unregister))
        .route(
            "/WorkerService/GetNextSubTask",
            post(worker::get_next_sub_task),
        )
        .route(
            "/WorkerService/ReportSubTaskStatus",
            post(worker::report_sub_task_status),
        )
        .route(
            "/WorkerService/EmitSessionEvent",
            post(worker::emit_session_event),
        )
        // SecretsService
        .route("/SecretsService/Send", post(secrets::send))
        .route("/SecretsService/Clear", post(secrets::clear))
        .route("/SecretsService/Get", post(secrets::get))
}
