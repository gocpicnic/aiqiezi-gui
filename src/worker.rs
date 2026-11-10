/*
  src/worker.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use std::fmt;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use aiqiezi_client::aiqiezi_proto::{Request, Response, PROTOCOL_VERSION};
use aiqiezi_client::{local, ClientConfig};
use serde_json::{json, Value};
use tokio::sync::mpsc::{self as tokio_mpsc, UnboundedReceiver, UnboundedSender};
use tracing::{info, warn};

use crate::daemon_process;

/// Identifies one command sent to a worker.
///
/// Every result event carries the same id, so the UI can correlate it with the command that
/// produced it. This is also the foundation for separate library and transfer worker lanes: each
/// lane returns ids from its own handle, and the UI can keep several requests in flight.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RequestId(u64);

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub enum Command {
    EnsureDaemon { endpoint_id: Option<String> },
    Status,
    SetServer(String),
    Login(String),
    ChangePassword { old: String, new: String },
    ServerInfo,
    Reconnect,
    ShutdownDaemonAndExit,
}

impl Command {
    /// A stable label for logs and pending-request display; never contains secret input.
    pub fn label(&self) -> &'static str {
        match self {
            Self::EnsureDaemon { .. } => "ensure-daemon",
            Self::Status => "status",
            Self::SetServer(_) => "set-server",
            Self::Login(_) => "login",
            Self::ChangePassword { .. } => "change-password",
            Self::ServerInfo => "server-info",
            Self::Reconnect => "reconnect",
            Self::ShutdownDaemonAndExit => "shutdown-daemon",
        }
    }
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnsureDaemon { endpoint_id } => f
                .debug_struct("EnsureDaemon")
                .field("endpoint_supplied", &endpoint_id.is_some())
                .finish(),
            Self::Status => f.write_str("Status"),
            Self::SetServer(endpoint_id) => f.debug_tuple("SetServer").field(endpoint_id).finish(),
            Self::Login(_) => f.write_str("Login(<redacted>)"),
            Self::ChangePassword { .. } => f.write_str("ChangePassword(<redacted>)"),
            Self::ServerInfo => f.write_str("ServerInfo"),
            Self::Reconnect => f.write_str("Reconnect"),
            Self::ShutdownDaemonAndExit => f.write_str("ShutdownDaemonAndExit"),
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Fatal(String),
    EnsureDaemon {
        id: RequestId,
        result: Result<StatusInfo, String>,
    },
    Status {
        id: RequestId,
        result: Result<StatusInfo, String>,
    },
    DaemonStopped {
        id: RequestId,
        result: Result<(), String>,
    },
    Response {
        id: RequestId,
        label: String,
        response: Response,
    },
}

impl Event {
    pub fn request_id(&self) -> Option<RequestId> {
        match self {
            Self::Fatal(_) => None,
            Self::EnsureDaemon { id, .. }
            | Self::Status { id, .. }
            | Self::DaemonStopped { id, .. }
            | Self::Response { id, .. } => Some(*id),
        }
    }
}

struct Envelope {
    id: RequestId,
    command: Command,
}

type Wake = Arc<dyn Fn() + Send + Sync>;

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

/// Handle to the long-lived control worker lane.
///
/// The command channel is serialized: one command runs at a time. Future library and transfer
/// work must use separate lane handles rather than queueing behind this one.
pub struct WorkerHandle {
    commands: UnboundedSender<Envelope>,
    events: Receiver<Event>,
    next_request_id: u64,
}

impl WorkerHandle {
    /// Start the control worker lane.
    ///
    /// `wake` is called after every event is queued. The GUI passes an `egui::Context` repaint
    /// closure, so results wake the UI instead of relying on a polling timer.
    pub fn start(
        config: ClientConfig,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> Result<Self, String> {
        let (command_tx, command_rx) = tokio_mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::channel();
        let wake: Wake = Arc::new(wake);
        let worker_wake = Arc::clone(&wake);

        std::thread::Builder::new()
            .name("aiqiezi-gui-worker".to_string())
            .spawn(move || worker_thread(config, command_rx, event_tx, worker_wake))
            .map_err(|error| format!("spawn GUI worker thread: {error}"))?;

        info!("GUI control worker started");
        Ok(Self {
            commands: command_tx,
            events: event_rx,
            next_request_id: 1,
        })
    }

    /// Queue a command and return the id its result event will carry.
    ///
    /// `None` means the worker is no longer running.
    pub fn submit(&mut self, command: Command) -> Option<RequestId> {
        let id = RequestId(self.next_request_id);
        self.next_request_id = self.next_request_id.saturating_add(1);
        if self.commands.send(Envelope { id, command }).is_err() {
            warn!("GUI worker is not running");
            return None;
        }
        Some(id)
    }

    pub fn try_recv(&self) -> Option<Event> {
        self.events.try_recv().ok()
    }
}

fn worker_thread(
    config: ClientConfig,
    mut commands: UnboundedReceiver<Envelope>,
    events: Sender<Event>,
    wake: Wake,
) {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            send_event(
                &events,
                &wake,
                Event::Fatal(format!("failed to start worker runtime: {error}")),
            );
            return;
        }
    };

    runtime.block_on(async move {
        while let Some(envelope) = commands.recv().await {
            let Envelope { id, command } = envelope;
            match command {
                Command::EnsureDaemon { endpoint_id } => {
                    let result = ensure_daemon(&config, endpoint_id, id).await;
                    send_event(&events, &wake, Event::EnsureDaemon { id, result });
                }
                Command::Status => {
                    let result = status_request(&config, id).await;
                    send_event(&events, &wake, Event::Status { id, result });
                }
                Command::SetServer(endpoint_id) => {
                    let result = local_request(
                        &config,
                        "set-server",
                        json!({"endpoint_id": endpoint_id}),
                        id,
                    )
                    .await;
                    send_response(&events, &wake, id, "set-server", result);
                }
                Command::Login(password) => {
                    let result =
                        local_request(&config, "login", json!({"password": password}), id).await;
                    send_response(&events, &wake, id, "login", result);
                }
                Command::ChangePassword { old, new } => {
                    let result = local_request(
                        &config,
                        "change-password",
                        json!({"old_password": old, "new_password": new}),
                        id,
                    )
                    .await;
                    send_response(&events, &wake, id, "change-password", result);
                }
                Command::ServerInfo => {
                    let result = local_request(&config, "server-info", json!({}), id).await;
                    send_response(&events, &wake, id, "server-info", result);
                }
                Command::Reconnect => {
                    let result = local_request(&config, "reconnect", json!({}), id).await;
                    send_response(&events, &wake, id, "reconnect", result);
                }
                Command::ShutdownDaemonAndExit => {
                    let result = stop_daemon(&config, id).await;
                    send_event(&events, &wake, Event::DaemonStopped { id, result });
                }
            }
        }

        info!("GUI worker stopped");
    });
}

async fn ensure_daemon(
    config: &ClientConfig,
    endpoint_id: Option<String>,
    id: RequestId,
) -> Result<StatusInfo, String> {
    if let Ok(status) = status_request(config, id).await {
        if let Some(endpoint_id) = endpoint_id.as_deref() {
            if status.server_id.as_deref() != Some(endpoint_id) {
                let response = local_request(
                    config,
                    "set-server",
                    json!({"endpoint_id": endpoint_id}),
                    id,
                )
                .await
                .map_err(|error| format!("{error:#}"))?;
                if !response.ok {
                    return Err(response_error(&response));
                }
                return status_request(config, id).await;
            }
        }
        return Ok(status);
    }

    let mut spawned = daemon_process::spawn_daemon(config, endpoint_id.as_deref())?;
    let deadline = Instant::now() + Duration::from_secs(20);

    loop {
        if let Ok(status) = status_request(config, id).await {
            return Ok(status);
        }

        match spawned.child.try_wait() {
            Ok(Some(status)) => {
                return Err(format!(
                    "client daemon exited during startup ({status}); log: {}",
                    spawned.log_path.display()
                ));
            }
            Ok(None) => {}
            Err(error) => {
                return Err(format!(
                    "could not check the client daemon process: {error}; log: {}",
                    spawned.log_path.display()
                ));
            }
        }

        if Instant::now() >= deadline {
            return Err(format!(
                "client daemon did not become ready within 20 seconds; log: {}",
                spawned.log_path.display()
            ));
        }

        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

async fn stop_daemon(config: &ClientConfig, id: RequestId) -> Result<(), String> {
    match local_request(config, "shutdown", json!({}), id).await {
        Ok(response) if response.ok => Ok(()),
        Ok(response) => Err(response_error(&response)),
        Err(error) => {
            if status_request(config, id).await.is_err() {
                Ok(())
            } else {
                Err(format!("{error:#}"))
            }
        }
    }
}

async fn status_request(config: &ClientConfig, id: RequestId) -> Result<StatusInfo, String> {
    let response = local_request(config, "status", json!({}), id)
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
    id: RequestId,
) -> anyhow::Result<Response> {
    let request = Request {
        version: PROTOCOL_VERSION,
        request_id: Some(format!("gui-{}-{id}", std::process::id())),
        command: command.to_string(),
        params,
    };

    local::request(&config.socket_path, &request).await
}

fn send_response(
    events: &Sender<Event>,
    wake: &Wake,
    id: RequestId,
    label: &str,
    result: anyhow::Result<Response>,
) {
    let response = match result {
        Ok(response) => response,
        Err(error) => Response::error(
            None,
            aiqiezi_client::aiqiezi_proto::error_codes::INTERNAL,
            format!("{error:#}"),
        ),
    };
    send_event(
        events,
        wake,
        Event::Response {
            id,
            label: label.to_string(),
            response,
        },
    );
}

fn send_event(events: &Sender<Event>, wake: &Wake, event: Event) {
    if events.send(event).is_ok() {
        wake();
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
    use super::{stop_daemon, Command, Event, RequestId, StatusInfo, WorkerHandle};
    use aiqiezi_client::aiqiezi_proto::{Request, Response};
    use aiqiezi_client::ClientConfig;
    use serde_json::json;

    #[cfg(unix)]
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    #[cfg(unix)]
    #[tokio::test]
    async fn stop_daemon_sends_shutdown() {
        let dir = tempfile::tempdir().unwrap();
        let config = ClientConfig::load(dir.path().to_path_buf(), dir.path().join("run")).unwrap();
        std::fs::create_dir_all(&config.runtime_dir).unwrap();
        let listener = tokio::net::UnixListener::bind(&config.socket_path).unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (read_half, mut write_half) = stream.into_split();
            let mut reader = BufReader::new(read_half);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let request: Request = serde_json::from_str(line.trim()).unwrap();
            assert_eq!(request.command, "shutdown");
            let response =
                Response::success(request.request_id, json!({"message": "shutting down"}));
            let mut encoded = serde_json::to_vec(&response).unwrap();
            encoded.push(b'\n');
            write_half.write_all(&encoded).await.unwrap();
            write_half.flush().await.unwrap();
        });

        stop_daemon(&config, RequestId(1)).await.unwrap();
        server.await.unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn stop_daemon_is_ok_when_daemon_is_not_running() {
        let dir = tempfile::tempdir().unwrap();
        let config = ClientConfig::load(dir.path().to_path_buf(), dir.path().join("run")).unwrap();

        stop_daemon(&config, RequestId(1)).await.unwrap();
    }

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

    #[test]
    fn command_debug_redacts_passwords() {
        let login = format!("{:?}", Command::Login("bootstrap-secret".into()));
        assert_eq!(login, "Login(<redacted>)");
        assert!(!login.contains("bootstrap-secret"));

        let change = format!(
            "{:?}",
            Command::ChangePassword {
                old: "old-secret".into(),
                new: "new-secret".into(),
            }
        );
        assert_eq!(change, "ChangePassword(<redacted>)");
        assert!(!change.contains("old-secret"));
        assert!(!change.contains("new-secret"));
    }

    #[test]
    fn command_labels_are_stable_and_public() {
        assert_eq!(Command::Status.label(), "status");
        assert_eq!(Command::ServerInfo.label(), "server-info");
        assert_eq!(
            Command::Login("secret".into()).label(),
            "login",
            "labels must not include secret input"
        );
    }

    #[test]
    fn result_events_carry_their_request_id() {
        let id = RequestId(42);
        let event = Event::Status {
            id,
            result: Err("offline".into()),
        };
        assert_eq!(event.request_id(), Some(id));

        let response = Event::Response {
            id,
            label: "status".into(),
            response: Response::success(None, json!({})),
        };
        assert_eq!(response.request_id(), Some(id));

        let fatal = Event::Fatal("worker stopped".into());
        assert_eq!(fatal.request_id(), None);
    }

    #[test]
    fn worker_assigns_unique_request_ids_and_detects_a_closed_queue() {
        let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();
        let (_event_tx, event_rx) = std::sync::mpsc::channel();
        let mut worker = WorkerHandle {
            commands: command_tx,
            events: event_rx,
            next_request_id: 1,
        };

        assert_eq!(worker.submit(Command::Status), Some(RequestId(1)));
        assert_eq!(worker.submit(Command::Status), Some(RequestId(2)));

        drop(command_rx);
        assert_eq!(worker.submit(Command::Status), None);
    }

    #[test]
    fn worker_returns_correlated_event_and_wakes_the_ui() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use std::time::{Duration, Instant};

        let dir = tempfile::tempdir().unwrap();
        let config = ClientConfig::load(dir.path().to_path_buf(), dir.path().join("run")).unwrap();
        let woke = Arc::new(AtomicBool::new(false));
        let worker_woke = Arc::clone(&woke);
        let mut worker = WorkerHandle::start(config, move || {
            worker_woke.store(true, Ordering::SeqCst);
        })
        .expect("start worker");

        let id = worker
            .submit(Command::ShutdownDaemonAndExit)
            .expect("worker accepts commands");
        let deadline = Instant::now() + Duration::from_secs(2);
        let event = loop {
            if let Some(event) = worker.try_recv() {
                break event;
            }
            assert!(Instant::now() < deadline, "worker did not answer");
            std::thread::sleep(Duration::from_millis(10));
        };

        assert_eq!(event.request_id(), Some(id));
        assert!(matches!(event, Event::DaemonStopped { result: Ok(()), .. }));
        assert!(
            woke.load(Ordering::SeqCst),
            "the worker must wake the UI after queuing an event"
        );
    }
}
