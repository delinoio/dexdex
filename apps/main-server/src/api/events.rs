//! EventStreamService - SSE subscription handler.

use axum::{
    Json,
    extract::State,
    response::sse::{Event, Sse},
};
use rpc_protocol::requests::SubscribeRequest;
use tokio_stream::{Stream, StreamExt, wrappers::BroadcastStream};

use crate::{error::AppResult, state::SharedState};

/// Subscribe to the event stream for a workspace.
///
/// Returns an SSE stream that sends `StreamEvent` objects as JSON.
pub async fn subscribe(
    State(state): State<SharedState>,
    Json(req): Json<SubscribeRequest>,
) -> AppResult<Sse<impl Stream<Item = Result<Event, axum::Error>>>> {
    let workspace_id = req.workspace_id.to_string();
    let receiver = state.broker.subscribe();

    let stream = BroadcastStream::new(receiver).filter_map(move |result| {
        let ws_id = workspace_id.clone();
        match result {
            Ok(event) if event.workspace_id == ws_id || event.workspace_id.is_empty() => {
                let data = serde_json::to_string(&event).unwrap_or_default();
                let sse_event = Event::default()
                    .event(format!("{:?}", event.event_type))
                    .data(data);
                Some(Ok(sse_event))
            }
            Ok(_) => None,
            Err(_) => None,
        }
    });

    Ok(Sse::new(stream))
}
