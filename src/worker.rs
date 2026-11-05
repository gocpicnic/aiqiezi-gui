/*
  src/worker.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use std::sync::mpsc::{self, Receiver, Sender};

use aiqiezi_client::aiqiezi_proto::{Request, Response, PROTOCOL_VERSION};
use aiqiezi_client::{local, ClientConfig};
use serde_json::{json, Value};
use tokio::sync::mpsc::{self as tokio_mpsc, UnboundedReceiver, UnboundedSender};
use tracing::{info, warn};

#[derive(Debug)]
pub enum Command {
    Status,
    SetServer(String),
    Login(String),
    ChangePassword { old: String, new: String },
    ServerInfo,
}

#[derive(Debug)]
pub enum Event {
    Fatal(String),
    Status(Result<StatusInfo, String>),
    Response { label: String, response: Response },
}

#[derive(Debug, Clone, Default)]
pub struct StatusInfo {
    pub client_id: String,
    pub server_id: Option<String>,
    pub socket: String,
    pub version: String,
}

impl StatusInfo {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        let object = value
            .as_object()
            .ok_or_else(|| "status response is not an object".to_string())?;

        let client_id = object
            .get("client_id")
            .and_then(Value::as_str)
            .ok_or_else(|| "status response is missing client_id".to_string())?
            .to_string();
        let server_id = object
            .get("server_id")
            .and_then(Value::as_str)
            .map(str::to_string);
        let socket = object
            .get("socket")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let version = object
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();

        Ok(Self {
            client_id,
            server_id,
            socket,
            version,
        })
    }
}

pub struct WorkerHandle {
    commands: UnboundedSender<Command>,
    events: Receiver<Event>,
}

impl WorkerHandle {
    pub fn start(config: ClientConfig) -> Self {
        let (command_tx, command_rx) = tokio_mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::channel();

        std::thread::Builder::new()
            .name("aiqiezi-gui-worker".to_string())
            .spawn(move || worker_thread(config, command_rx, event_tx))
            .expect("spawn GUI worker thread");

        Self {
            commands: command_tx,
            events: event_rx,
        }
    }

    pub fn send(&self, command: Command) {
        if self.commands.send(command).is_err() {
            warn!("GUI worker is not running");
        }
    }

    pub fn try_recv(&self) -> Option<Event> {
        self.events.try_recv().ok()
    }
}

fn worker_thread(
    config: ClientConfig,
    mut commands: UnboundedReceiver<Command>,
    events: Sender<Event>,
) {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            let _ = events.send(Event::Fatal(format!(
                "failed to start worker runtime: {error}"
            )));
            return;
        }
    };

    runtime.block_on(async move {
        while let Some(command) = commands.recv().await {
            match command {
                Command::Status => {
                    let result = status_request(&config).await;
                    let _ = events.send(Event::Status(result));
                }
                Command::SetServer(endpoint_id) => {
                    send_response(
                        &events,
                        "set-server",
                        local_request(&config, "set-server", json!({"endpoint_id": endpoint_id}))
                            .await,
                    );
                }
                Command::Login(password) => {
                    send_response(
                        &events,
                        "login",
                        local_request(&config, "login", json!({"password": password})).await,
                    );
                }
                Command::ChangePassword { old, new } => {
                    send_response(
                        &events,
                        "change-password",
                        local_request(
                            &config,
                            "change-password",
                            json!({"old_password": old, "new_password": new}),
                        )
                        .await,
                    );
                }
                Command::ServerInfo => {
                    send_response(
                        &events,
                        "server-info",
                        local_request(&config, "server-info", json!({})).await,
                    );
                }
            }
        }

        info!("GUI worker stopped");
    });
}

async fn status_request(config: &ClientConfig) -> Result<StatusInfo, String> {
    let response = local_request(config, "status", json!({}))
        .await
        .map_err(|error| format!("{error:#}"))?;
    if !response.ok {
        return Err(response_error(&response));
    }

    StatusInfo::from_value(response.data.as_ref().unwrap_or(&Value::Null))
}

async fn local_request(
    config: &ClientConfig,
    command: &str,
    params: Value,
) -> anyhow::Result<Response> {
    let request = Request {
        version: PROTOCOL_VERSION,
        request_id: Some(format!("gui-{}", std::process::id())),
        command: command.to_string(),
        params,
    };

    local::request(&config.socket_path, &request).await
}

fn send_response(events: &Sender<Event>, label: &str, result: anyhow::Result<Response>) {
    match result {
        Ok(response) => {
            let _ = events.send(Event::Response {
                label: label.to_string(),
                response,
            });
        }
        Err(error) => {
            let _ = events.send(Event::Response {
                label: label.to_string(),
                response: Response::error(
                    None,
                    aiqiezi_client::aiqiezi_proto::error_codes::INTERNAL,
                    format!("{error:#}"),
                ),
            });
        }
    }
}

fn response_error(response: &Response) -> String {
    response
        .error
        .as_ref()
        .map(|error| format!("{}: {}", error.code, error.message))
        .unwrap_or_else(|| "unknown local error".to_string())
}

#[cfg(test)]
mod tests {
    use super::StatusInfo;
    use serde_json::json;

    #[test]
    fn parses_status_payload() {
        let status = StatusInfo::from_value(&json!({
            "client_id": "client",
            "server_id": "server",
            "socket": "/tmp/client.sock",
            "version": "1.0",
        }))
        .unwrap();

        assert_eq!(status.client_id, "client");
        assert_eq!(status.server_id.as_deref(), Some("server"));
        assert_eq!(status.socket, "/tmp/client.sock");
        assert_eq!(status.version, "1.0");
    }

    #[test]
    fn parses_unconfigured_server() {
        let status = StatusInfo::from_value(&json!({
            "client_id": "client",
            "server_id": null,
        }))
        .unwrap();

        assert!(status.server_id.is_none());
    }

    #[test]
    fn rejects_missing_client_id() {
        assert!(StatusInfo::from_value(&json!({"server_id": "server"})).is_err());
    }

    #[test]
    fn rejects_non_object_status() {
        assert!(StatusInfo::from_value(&json!("not-an-object")).is_err());
    }
}
