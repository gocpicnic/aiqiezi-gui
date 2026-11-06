/*
  src/app.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use std::time::Duration;

use aiqiezi_client::aiqiezi_proto::Response;
use aiqiezi_client::{ClientConfig, EndpointId};
use eframe::egui::{self, Color32, RichText};
use serde_json::Value;

use crate::version;
use crate::worker::{Command, Event, StatusInfo, WorkerHandle};

const OK_COLOR: Color32 = Color32::from_rgb(80, 180, 100);
const ERROR_COLOR: Color32 = Color32::from_rgb(220, 90, 90);
const WARN_COLOR: Color32 = Color32::from_rgb(220, 170, 70);

enum ConnectionState {
    NeedsServer,
    Starting,
    Running(StatusInfo),
    Error(String),
}

struct Message {
    text: String,
    error: bool,
}

pub struct App {
    config_error: Option<String>,
    worker: Option<WorkerHandle>,
    connection: ConnectionState,
    setup_required: bool,
    close_requested: bool,
    server_id_input: String,
    password: String,
    old_password: String,
    new_password: String,
    confirm_password: String,
    message: Option<Message>,
    server_info: Option<Value>,
    busy: bool,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        match ClientConfig::from_env() {
            Ok(config) => {
                let setup_required = config.server_id.is_none();
                let worker = WorkerHandle::start(config);
                if setup_required {
                    Self {
                        config_error: None,
                        worker: Some(worker),
                        connection: ConnectionState::NeedsServer,
                        setup_required: true,
                        close_requested: false,
                        server_id_input: String::new(),
                        password: String::new(),
                        old_password: String::new(),
                        new_password: String::new(),
                        confirm_password: String::new(),
                        message: None,
                        server_info: None,
                        busy: false,
                    }
                } else {
                    worker.send(Command::EnsureDaemon { endpoint_id: None });
                    Self {
                        config_error: None,
                        worker: Some(worker),
                        connection: ConnectionState::Starting,
                        setup_required: false,
                        close_requested: false,
                        server_id_input: String::new(),
                        password: String::new(),
                        old_password: String::new(),
                        new_password: String::new(),
                        confirm_password: String::new(),
                        message: None,
                        server_info: None,
                        busy: true,
                    }
                }
            }
            Err(error) => Self {
                config_error: Some(format!("{error:#}")),
                worker: None,
                connection: ConnectionState::Error("client configuration failed".to_string()),
                setup_required: false,
                close_requested: false,
                server_id_input: String::new(),
                password: String::new(),
                old_password: String::new(),
                new_password: String::new(),
                confirm_password: String::new(),
                message: None,
                server_info: None,
                busy: false,
            },
        }
    }

    fn poll_events(&mut self) {
        let Some(worker) = self.worker.as_ref() else {
            return;
        };

        let events: Vec<Event> = std::iter::from_fn(|| worker.try_recv()).collect();
        for event in events {
            match event {
                Event::Fatal(error) => {
                    self.busy = false;
                    self.connection = ConnectionState::Error(error.clone());
                    self.message = Some(Message {
                        text: error,
                        error: true,
                    });
                }
                Event::EnsureDaemon(result) => {
                    self.busy = false;
                    match result {
                        Ok(status) => self.apply_status(status),
                        Err(error) => {
                            self.message = Some(Message {
                                text: error.clone(),
                                error: true,
                            });
                            if self.setup_required {
                                self.connection = ConnectionState::NeedsServer;
                            } else {
                                self.connection = ConnectionState::Error(error);
                            }
                        }
                    }
                }
                Event::Status(result) => {
                    self.busy = false;
                    match result {
                        Ok(status) => self.apply_status(status),
                        Err(error) => {
                            self.message = Some(Message {
                                text: error.clone(),
                                error: true,
                            });
                            if self.setup_required {
                                self.connection = ConnectionState::NeedsServer;
                            } else {
                                self.connection = ConnectionState::Error(error);
                            }
                        }
                    }
                }
                Event::DaemonStopped(result) => {
                    self.busy = false;
                    match result {
                        Ok(()) => {
                            self.close_requested = true;
                        }
                        Err(error) => {
                            self.message = Some(Message {
                                text: format!("could not stop client daemon: {error}"),
                                error: true,
                            });
                        }
                    }
                }
                Event::Response { label, response } => {
                    self.busy = false;
                    self.handle_response(&label, response);
                }
            }
        }
    }

    fn apply_status(&mut self, status: StatusInfo) {
        if let Some(server_id) = &status.server_id {
            if self.server_id_input.is_empty() {
                self.server_id_input = server_id.clone();
            }
            self.setup_required = false;
            self.connection = ConnectionState::Running(status);
        } else {
            self.setup_required = true;
            self.connection = ConnectionState::NeedsServer;
        }
    }

    fn handle_response(&mut self, label: &str, response: Response) {
        if !response.ok {
            self.message = Some(Message {
                text: response_error(&response),
                error: true,
            });
            return;
        }

        match label {
            "server-info" => {
                self.server_info = response.data;
                self.message = Some(Message {
                    text: "server-info received".to_string(),
                    error: false,
                });
            }
            "set-server" => {
                self.message = Some(Message {
                    text: "server EndpointId saved".to_string(),
                    error: false,
                });
                self.refresh_status();
            }
            "login" => {
                self.password.clear();
                self.message = Some(Message {
                    text: "login successful".to_string(),
                    error: false,
                });
                self.refresh_status();
            }
            "change-password" => {
                self.old_password.clear();
                self.new_password.clear();
                self.confirm_password.clear();
                self.message = Some(Message {
                    text: "password changed".to_string(),
                    error: false,
                });
                self.refresh_status();
            }
            _ => {
                self.message = Some(Message {
                    text: "request completed".to_string(),
                    error: false,
                });
            }
        }
    }

    fn send(&mut self, command: Command) {
        if let Some(worker) = &self.worker {
            worker.send(command);
            self.busy = true;
        }
    }

    fn refresh_status(&mut self) {
        self.send(Command::Status);
    }

    fn draw_header(&mut self, ui: &mut egui::Ui) -> Option<Command> {
        let mut pending = None;
        egui::Panel::top("header").show(ui, |ui| {
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.heading("aiqiezi");
                ui.label(RichText::new("photo/video archive").weak());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add_enabled(!self.busy, egui::Button::new("Exit and stop client"))
                        .on_hover_text("Stop the local client daemon and exit")
                        .clicked()
                    {
                        if self.worker.is_some() {
                            pending = Some(Command::ShutdownDaemonAndExit);
                        } else {
                            self.close_requested = true;
                        }
                    }
                    ui.label(RichText::new(format!("v{}", version::VERSION)).weak());
                });
            });
            ui.add_space(6.0);
        });
        pending
    }

    fn draw_status(&self, ui: &mut egui::Ui) {
        match &self.connection {
            ConnectionState::NeedsServer => {
                ui.label(
                    RichText::new("Server setup required")
                        .color(WARN_COLOR)
                        .strong(),
                );
            }
            ConnectionState::Starting => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Starting the local client daemon...");
                });
            }
            ConnectionState::Error(error) => {
                ui.label(
                    RichText::new("Client daemon is not available")
                        .color(ERROR_COLOR)
                        .strong(),
                );
                if let Some(config_error) = &self.config_error {
                    ui.label(RichText::new(config_error).color(ERROR_COLOR));
                } else {
                    ui.label(RichText::new(error).color(ERROR_COLOR));
                    ui.label("Use Refresh or start the daemon from the controls below.");
                }
            }
            ConnectionState::Running(status) => {
                ui.label(
                    RichText::new("Client daemon connected")
                        .color(OK_COLOR)
                        .strong(),
                );
                egui::Grid::new("status_grid")
                    .num_columns(2)
                    .spacing([12.0, 6.0])
                    .show(ui, |ui| {
                        ui.label("Client EndpointId");
                        ui.monospace(&status.client_id);
                        ui.end_row();

                        ui.label("Server EndpointId");
                        match &status.server_id {
                            Some(server_id) => ui.monospace(server_id),
                            None => ui.label(RichText::new("not configured").color(WARN_COLOR)),
                        };
                        ui.end_row();

                        ui.label("Local socket");
                        ui.monospace(&status.socket);
                        ui.end_row();

                        ui.label("Version");
                        ui.monospace(&status.version);
                        ui.end_row();
                    });
            }
        }
    }

    fn draw_setup(&mut self, ui: &mut egui::Ui) -> Option<Command> {
        let mut pending = None;
        ui.add_space(12.0);
        ui.heading("Connect to aiqiezi-server");
        ui.label(
            "Enter the aiqiezi-server EndpointId. The GUI will start the local client \
             daemon after saving it.",
        );
        ui.add_space(6.0);
        ui.label("Get it on the server host with:");
        ui.monospace("aiqiezi-server whoami");
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.label("Server EndpointId");
            ui.add(
                egui::TextEdit::singleline(&mut self.server_id_input)
                    .desired_width(560.0)
                    .hint_text("server EndpointId"),
            );
            if ui
                .add_enabled(
                    !self.busy && !self.server_id_input.trim().is_empty(),
                    egui::Button::new("Save and start client"),
                )
                .clicked()
            {
                match self.server_id_input.trim().parse::<EndpointId>() {
                    Ok(endpoint_id) => {
                        self.message = None;
                        self.connection = ConnectionState::Starting;
                        pending = Some(Command::EnsureDaemon {
                            endpoint_id: Some(endpoint_id.to_string()),
                        });
                    }
                    Err(error) => {
                        self.message = Some(Message {
                            text: format!("invalid aiqiezi-server EndpointId: {error}"),
                            error: true,
                        });
                    }
                }
            }
            if self.busy {
                ui.spinner();
            }
        });

        if let Some(message) = &self.message {
            ui.add_space(8.0);
            ui.label(RichText::new(&message.text).color(if message.error {
                ERROR_COLOR
            } else {
                OK_COLOR
            }));
        }

        pending
    }

    fn draw_controls(&mut self, ui: &mut egui::Ui) -> Option<Command> {
        if matches!(self.connection, ConnectionState::NeedsServer) {
            return self.draw_setup(ui);
        }

        if matches!(self.connection, ConnectionState::Starting) {
            return None;
        }

        let connected = matches!(self.connection, ConnectionState::Running(_));
        let mut pending = None;

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui
                .add_enabled(
                    self.worker.is_some() && !self.busy,
                    egui::Button::new("Refresh"),
                )
                .clicked()
            {
                pending = Some(Command::Status);
            }
            if ui
                .add_enabled(
                    self.worker.is_some() && !self.busy && !self.setup_required && !connected,
                    egui::Button::new("Start client daemon"),
                )
                .clicked()
            {
                pending = Some(Command::EnsureDaemon { endpoint_id: None });
            }
            if ui
                .add_enabled(connected && !self.busy, egui::Button::new("Server info"))
                .clicked()
            {
                pending = Some(Command::ServerInfo);
            }
            if self.busy {
                ui.spinner();
            }
        });

        if !connected {
            if let Some(message) = &self.message {
                ui.add_space(8.0);
                ui.label(RichText::new(&message.text).color(if message.error {
                    ERROR_COLOR
                } else {
                    OK_COLOR
                }));
            }
            return pending;
        }

        ui.separator();
        ui.heading("Server");
        ui.horizontal(|ui| {
            ui.label("EndpointId");
            ui.add(
                egui::TextEdit::singleline(&mut self.server_id_input)
                    .desired_width(560.0)
                    .hint_text("aiqiezi-server EndpointId"),
            );
            if ui
                .add_enabled(
                    !self.busy && !self.server_id_input.trim().is_empty(),
                    egui::Button::new("Save"),
                )
                .clicked()
            {
                pending = Some(Command::SetServer(self.server_id_input.trim().to_string()));
            }
        });

        ui.separator();
        ui.heading("Login");
        ui.horizontal(|ui| {
            ui.label("Password");
            ui.add(
                egui::TextEdit::singleline(&mut self.password)
                    .password(true)
                    .desired_width(320.0),
            );
            if ui
                .add_enabled(
                    !self.busy && !self.password.is_empty(),
                    egui::Button::new("Login"),
                )
                .clicked()
            {
                pending = Some(Command::Login(self.password.clone()));
            }
        });
        ui.label(
            RichText::new(
                "The first login requires the bootstrap password, then a password change.",
            )
            .weak(),
        );

        ui.separator();
        ui.heading("Change password");
        egui::Grid::new("change_password_grid")
            .num_columns(2)
            .spacing([12.0, 6.0])
            .show(ui, |ui| {
                ui.label("Current password");
                ui.add(
                    egui::TextEdit::singleline(&mut self.old_password)
                        .password(true)
                        .desired_width(320.0),
                );
                ui.end_row();

                ui.label("New password");
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_password)
                        .password(true)
                        .desired_width(320.0),
                );
                ui.end_row();

                ui.label("Confirm new password");
                ui.add(
                    egui::TextEdit::singleline(&mut self.confirm_password)
                        .password(true)
                        .desired_width(320.0),
                );
                ui.end_row();
            });

        if ui
            .add_enabled(
                !self.busy
                    && !self.old_password.is_empty()
                    && !self.new_password.is_empty()
                    && !self.confirm_password.is_empty(),
                egui::Button::new("Change password"),
            )
            .clicked()
        {
            if self.new_password != self.confirm_password {
                self.message = Some(Message {
                    text: "new passwords do not match".to_string(),
                    error: true,
                });
            } else {
                pending = Some(Command::ChangePassword {
                    old: self.old_password.clone(),
                    new: self.new_password.clone(),
                });
            }
        }

        if let Some(message) = &self.message {
            ui.add_space(8.0);
            ui.label(RichText::new(&message.text).color(if message.error {
                ERROR_COLOR
            } else {
                OK_COLOR
            }));
        }

        if let Some(info) = &self.server_info {
            ui.separator();
            ui.heading("server-info");
            egui::ScrollArea::vertical()
                .max_height(240.0)
                .show(ui, |ui| {
                    if let Ok(text) = serde_json::to_string_pretty(info) {
                        ui.monospace(text);
                    }
                });
        }

        pending
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_events();
        let mut pending = self.draw_header(ui);

        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.draw_status(ui);
                ui.add_space(12.0);
                if let Some(command) = self.draw_controls(ui) {
                    pending = Some(command);
                }
            });
        });

        if let Some(command) = pending {
            self.send(command);
        }

        if self.close_requested {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if self.busy {
            ui.ctx().request_repaint_after(Duration::from_millis(100));
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
