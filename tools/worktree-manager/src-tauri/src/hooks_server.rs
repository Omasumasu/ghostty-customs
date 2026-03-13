use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::types::HookEvent;

#[derive(Clone)]
pub struct HooksState {
    pub sender: broadcast::Sender<HookEvent>,
}

async fn handle_hook(
    State(state): State<Arc<HooksState>>,
    Json(body): Json<serde_json::Value>,
) -> StatusCode {
    let event = HookEvent::from_value(body);
    // Ignore send errors (no receivers yet)
    let _ = state.sender.send(event);
    StatusCode::OK
}

/// Start the hooks HTTP server on an auto-assigned port.
/// Returns the port it bound to and a handle to the broadcast sender.
pub async fn start_hooks_server() -> Result<(u16, broadcast::Sender<HookEvent>), String> {
    let (sender, _) = broadcast::channel::<HookEvent>(256);
    let state = Arc::new(HooksState {
        sender: sender.clone(),
    });

    let app = Router::new()
        .route("/hooks/notification", post(handle_hook))
        .route("/hooks/tool", post(handle_hook))
        .route("/hooks/lifecycle", post(handle_hook))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind: {e}"))?;
    let local_addr = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local addr: {e}"))?;
    let port = local_addr.port();

    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });

    Ok((port, sender))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hooks_server_starts() {
        let (port, _sender) = start_hooks_server().await.unwrap();
        assert!(port > 0);
    }

    #[tokio::test]
    async fn test_hooks_server_receives_event() {
        let (port, sender) = start_hooks_server().await.unwrap();
        let mut rx = sender.subscribe();

        let client = reqwest::Client::new();
        let resp = client
            .post(format!("http://127.0.0.1:{port}/hooks/notification"))
            .json(&serde_json::json!({
                "type": "notification",
                "message": "hello"
            }))
            .send()
            .await
            .unwrap();

        assert_eq!(resp.status(), 200);

        let event = rx.recv().await.unwrap();
        match event {
            HookEvent::Notification { message, .. } => {
                assert_eq!(message, "hello");
            }
            _ => panic!("Expected Notification event"),
        }
    }

    #[tokio::test]
    async fn test_hooks_server_tool_endpoint() {
        let (port, sender) = start_hooks_server().await.unwrap();
        let mut rx = sender.subscribe();

        let client = reqwest::Client::new();
        client
            .post(format!("http://127.0.0.1:{port}/hooks/tool"))
            .json(&serde_json::json!({
                "type": "tool_use",
                "tool_name": "Read",
                "session_id": "s1"
            }))
            .send()
            .await
            .unwrap();

        let event = rx.recv().await.unwrap();
        match event {
            HookEvent::ToolUse { tool_name, .. } => {
                assert_eq!(tool_name, "Read");
            }
            _ => panic!("Expected ToolUse event"),
        }
    }

    #[tokio::test]
    async fn test_hooks_server_lifecycle_endpoint() {
        let (port, sender) = start_hooks_server().await.unwrap();
        let mut rx = sender.subscribe();

        let client = reqwest::Client::new();
        client
            .post(format!("http://127.0.0.1:{port}/hooks/lifecycle"))
            .json(&serde_json::json!({
                "type": "start",
                "session_id": "s2",
                "cwd": "/tmp"
            }))
            .send()
            .await
            .unwrap();

        let event = rx.recv().await.unwrap();
        match event {
            HookEvent::Start { session_id, cwd } => {
                assert_eq!(session_id, Some("s2".to_string()));
                assert_eq!(cwd, Some("/tmp".to_string()));
            }
            _ => panic!("Expected Start event"),
        }
    }
}
