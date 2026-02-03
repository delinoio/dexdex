//! Local agent executor for single-process mode.
//!
//! This module runs AI coding agents directly on the local machine
//! without Docker isolation.

use std::{path::PathBuf, sync::Arc};

use chrono::Utc;
use coding_agents::{create_agent, AgentConfig, NormalizedEvent};
use entities::{AgentSession, AiAgentType, UnitTask, UnitTaskStatus};
use task_store::{SqliteTaskStore, TaskStore};
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::error::AppResult;

/// Local agent executor that runs agents directly on the local machine.
pub struct LocalExecutor {
    task_store: Arc<SqliteTaskStore>,
}

impl LocalExecutor {
    /// Creates a new local executor.
    pub fn new(task_store: Arc<SqliteTaskStore>) -> Self {
        Self { task_store }
    }

    /// Executes an agent for a unit task.
    ///
    /// This method:
    /// 1. Creates an agent session
    /// 2. Runs the agent
    /// 3. Collects output and stores it in the session
    /// 4. Updates task status based on result
    pub async fn execute_unit_task(
        &self,
        unit_task: &UnitTask,
        working_dir: PathBuf,
        agent_type: AiAgentType,
        agent_model: Option<String>,
    ) -> AppResult<()> {
        let agent_task_id = unit_task.agent_task_id;
        let unit_task_id = unit_task.id;

        info!(
            "Starting agent execution for unit task {} (agent task {})",
            unit_task_id, agent_task_id
        );

        // Create an agent session
        let mut session = AgentSession::new(agent_task_id, agent_type);
        session.started_at = Some(Utc::now());
        if let Some(ref model) = agent_model {
            session = session.with_model(model);
        }

        // Store the session
        let session = self.task_store.create_agent_session(session).await?;
        let session_id = session.id;

        info!(
            "Created agent session {} for task {}",
            session_id, unit_task_id
        );

        // Run the agent in a spawned task
        let task_store = self.task_store.clone();
        let prompt = unit_task.prompt.clone();
        let working_dir_str = working_dir.to_string_lossy().to_string();

        tokio::spawn(async move {
            let result = run_agent_with_output(
                agent_type,
                working_dir_str,
                prompt,
                agent_model,
                session_id,
                agent_task_id,
                unit_task_id,
                task_store.clone(),
            )
            .await;

            // Update unit task status based on result
            if let Err(e) =
                update_task_status_after_execution(&task_store, unit_task_id, result.is_ok()).await
            {
                error!("Failed to update task status: {}", e);
            }
        });

        Ok(())
    }
}

/// Runs the agent and collects output.
async fn run_agent_with_output(
    agent_type: AiAgentType,
    working_dir: String,
    prompt: String,
    model: Option<String>,
    session_id: Uuid,
    _agent_task_id: Uuid,
    unit_task_id: Uuid,
    task_store: Arc<SqliteTaskStore>,
) -> AppResult<()> {
    info!("Running {} agent in {}", agent_type.as_str(), working_dir);

    // Create the agent
    let agent = create_agent(agent_type);

    // Build config
    let mut config = AgentConfig::new(agent_type, &working_dir, &prompt);
    if let Some(ref m) = model {
        config = config.with_model(m);
    }

    // Create channel for events
    let (event_tx, mut event_rx) = mpsc::channel::<NormalizedEvent>(1024);

    // Start collecting output
    let output_collector = tokio::spawn({
        let task_store = task_store.clone();
        async move {
            let mut output_log = String::new();
            let mut last_update = std::time::Instant::now();

            while let Some(event) = event_rx.recv().await {
                // Format event for output log
                let line = format_event_for_log(&event);
                if !line.is_empty() {
                    output_log.push_str(&line);
                    output_log.push('\n');
                }

                // Periodically update the session in the database (every 5 seconds)
                if last_update.elapsed() > std::time::Duration::from_secs(5) {
                    if let Err(e) =
                        update_session_output(&task_store, session_id, &output_log).await
                    {
                        warn!("Failed to update session output: {}", e);
                    }
                    last_update = std::time::Instant::now();
                }
            }

            // Final update
            output_log
        }
    });

    // Run the agent
    let result = agent.run(config, event_tx, None).await;

    // Wait for output collection to complete
    let output_log = output_collector.await.unwrap_or_default();

    // Update session with final output and completion time
    if let Ok(Some(mut session)) = task_store.get_agent_session(session_id).await {
        session.output_log = Some(output_log);
        session.completed_at = Some(Utc::now());
        if let Err(e) = task_store.update_agent_session(session).await {
            error!("Failed to update agent session: {}", e);
        }
    }

    match result {
        Ok(()) => {
            info!("Agent completed successfully for task {}", unit_task_id);
            Ok(())
        }
        Err(e) => {
            error!("Agent failed for task {}: {}", unit_task_id, e);
            Err(crate::error::AppError::Internal(format!(
                "Agent execution failed: {}",
                e
            )))
        }
    }
}

/// Formats an event for the output log.
fn format_event_for_log(event: &NormalizedEvent) -> String {
    match event {
        NormalizedEvent::TextOutput { content, .. } => content.clone(),
        NormalizedEvent::ErrorOutput { content } => format!("[ERROR] {}", content),
        NormalizedEvent::ToolUse { tool_name, input } => {
            format!(
                "[TOOL] {} - {}",
                tool_name,
                serde_json::to_string(input).unwrap_or_default()
            )
        }
        NormalizedEvent::ToolResult {
            tool_name,
            is_error,
            ..
        } => {
            if *is_error {
                format!("[TOOL RESULT ERROR] {}", tool_name)
            } else {
                format!("[TOOL RESULT] {}", tool_name)
            }
        }
        NormalizedEvent::FileChange {
            path, change_type, ..
        } => format!("[FILE {:?}] {}", change_type, path),
        NormalizedEvent::CommandExecution {
            command, exit_code, ..
        } => {
            if let Some(code) = exit_code {
                format!("[CMD] {} (exit: {})", command, code)
            } else {
                format!("[CMD] {}", command)
            }
        }
        NormalizedEvent::Thinking { content } => format!("[THINKING] {}", content),
        NormalizedEvent::SessionStart { agent_type, .. } => {
            format!("[SESSION START] {}", agent_type)
        }
        NormalizedEvent::SessionEnd { success, error } => {
            if *success {
                "[SESSION END] Success".to_string()
            } else {
                format!(
                    "[SESSION END] Failed: {}",
                    error.as_deref().unwrap_or("Unknown error")
                )
            }
        }
        NormalizedEvent::AskUserQuestion { question, .. } => {
            format!("[QUESTION] {}", question)
        }
        NormalizedEvent::UserResponse { response } => format!("[RESPONSE] {}", response),
        NormalizedEvent::Raw { content } => content.clone(),
    }
}

/// Updates session output in the database.
async fn update_session_output(
    task_store: &SqliteTaskStore,
    session_id: Uuid,
    output: &str,
) -> AppResult<()> {
    if let Some(mut session) = task_store.get_agent_session(session_id).await? {
        session.output_log = Some(output.to_string());
        task_store.update_agent_session(session).await?;
    }
    Ok(())
}

/// Updates the unit task status after agent execution.
async fn update_task_status_after_execution(
    task_store: &SqliteTaskStore,
    unit_task_id: Uuid,
    success: bool,
) -> AppResult<()> {
    if let Some(mut task) = task_store.get_unit_task(unit_task_id).await? {
        task.status = if success {
            UnitTaskStatus::InReview
        } else {
            // Keep in progress on failure so user can retry
            UnitTaskStatus::InProgress
        };
        task.updated_at = Utc::now();
        task_store.update_unit_task(task).await?;
        info!(
            "Updated task {} status to {:?}",
            unit_task_id,
            if success {
                UnitTaskStatus::InReview
            } else {
                UnitTaskStatus::InProgress
            }
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_event_text_output() {
        let event = NormalizedEvent::TextOutput {
            content: "Hello, world!".to_string(),
            stream: false,
        };
        assert_eq!(format_event_for_log(&event), "Hello, world!");
    }

    #[test]
    fn test_format_event_error() {
        let event = NormalizedEvent::ErrorOutput {
            content: "Something went wrong".to_string(),
        };
        assert_eq!(format_event_for_log(&event), "[ERROR] Something went wrong");
    }
}
