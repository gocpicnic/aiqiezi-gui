/*
  src/app.rs
  Copyright 2026 aiqiezi. All rights reserved.
  Use of this source code is governed by the AGPL-3.0
  license that can be found in the LICENSE file.
*/

use std::time::Duration;

use aiqiezi_client::aiqiezi_proto::{error_codes, Response};
use aiqiezi_client::{ClientConfig, EndpointId};
use eframe::egui::{self, RichText};
use serde_json::Value;

use crate::theme;
use crate::version;
use crate::worker::{Command, Event, StatusInfo, WorkerHandle};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ConnectionState {
    NeedsServer,
    Starting,
    Running,
    Error,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NavItem {
    AllPhotos,
    Videos,
    RecentlyAdded,
    Favorites,
    Albums,
    Folders,
    Tags,
    RecentlyDeleted,
}

impl NavItem {
    fn label(self) -> &'static str {
        match self {
            Self::AllPhotos => "All Photos",
            Self::Videos => "Videos",
            Self::RecentlyAdded => "Recently Added",
            Self::Favorites => "Favorites",
            Self::Albums => "Albums",
            Self::Folders => "Folders",
            Self::Tags => "Tags",
            Self::RecentlyDeleted => "Recently Deleted",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InspectorTab {
    Info,
    Server,
}

#[derive(Clone, Copy)]
enum MenuAction {
    Refresh,
    Exit,
    ToggleSidebar,
    ToggleInspector,
    GridView,
    ListView,
    ClearSearch,
    Login,
    ChangePassword,
    ServerInfo,
    ChangeEndpoint,
    Reconnect,
    About,
}

struct Message {
    text: String,
    error: bool,
}

pub struct App {
    config_error: Option<String>,
    worker: Option<WorkerHandle>,
    connection: ConnectionState,
    connection_error: Option<String>,
    status: Option<StatusInfo>,
    setup_required: bool,
    close_requested: bool,
    busy: bool,
    message: Option<Message>,
    server_id_input: String,
    server_info: Option<Value>,

    nav: NavItem,
    inspector_tab: InspectorTab,
    show_sidebar: bool,
    show_inspector: bool,
    grid_view: bool,
    search: String,

    login_open: bool,
    login_password: String,
    change_password_open: bool,
    old_password: String,
    new_password: String,
    confirm_password: String,
    endpoint_open: bool,
    server_info_open: bool,
    about_open: bool,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);

        match ClientConfig::from_env() {
            Ok(config) => {
                let setup_required = config.server_id.is_none();
                let worker = WorkerHandle::start(config);
                if setup_required {
                    Self {
                        config_error: None,
                        worker: Some(worker),
                        connection: ConnectionState::NeedsServer,
                        connection_error: None,
                        status: None,
                        setup_required: true,
                        close_requested: false,
                        busy: false,
                        message: None,
                        server_id_input: String::new(),
                        server_info: None,
                        nav: NavItem::AllPhotos,
                        inspector_tab: InspectorTab::Info,
                        show_sidebar: true,
                        show_inspector: true,
                        grid_view: true,
                        search: String::new(),
                        login_open: false,
                        login_password: String::new(),
                        change_password_open: false,
                        old_password: String::new(),
                        new_password: String::new(),
                        confirm_password: String::new(),
                        endpoint_open: false,
                        server_info_open: false,
                        about_open: false,
                    }
                } else {
                    worker.send(Command::EnsureDaemon { endpoint_id: None });
                    Self {
                        config_error: None,
                        worker: Some(worker),
                        connection: ConnectionState::Starting,
                        connection_error: None,
                        status: None,
                        setup_required: false,
                        close_requested: false,
                        busy: true,
                        message: None,
                        server_id_input: String::new(),
                        server_info: None,
                        nav: NavItem::AllPhotos,
                        inspector_tab: InspectorTab::Info,
                        show_sidebar: true,
                        show_inspector: true,
                        grid_view: true,
                        search: String::new(),
                        login_open: false,
                        login_password: String::new(),
                        change_password_open: false,
                        old_password: String::new(),
                        new_password: String::new(),
                        confirm_password: String::new(),
                        endpoint_open: false,
                        server_info_open: false,
                        about_open: false,
                    }
                }
            }
            Err(error) => Self {
                config_error: Some(format!("{error:#}")),
                worker: None,
                connection: ConnectionState::Error,
                connection_error: Some("client configuration failed".to_string()),
                status: None,
                setup_required: false,
                close_requested: false,
                busy: false,
                message: None,
                server_id_input: String::new(),
                server_info: None,
                nav: NavItem::AllPhotos,
                inspector_tab: InspectorTab::Info,
                show_sidebar: true,
                show_inspector: true,
                grid_view: true,
                search: String::new(),
                login_open: false,
                login_password: String::new(),
                change_password_open: false,
                old_password: String::new(),
                new_password: String::new(),
                confirm_password: String::new(),
                endpoint_open: false,
                server_info_open: false,
                about_open: false,
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
                    self.connection = ConnectionState::Error;
                    self.connection_error = Some(error.clone());
                    self.message = Some(Message {
                        text: error,
                        error: true,
                    });
                }
                Event::EnsureDaemon(result) => {
                    self.busy = false;
                    let was_setup = self.setup_required;
                    match result {
                        Ok(status) => {
                            self.apply_status(status);
                            if was_setup && !self.setup_required {
                                self.login_open = true;
                            }
                        }
                        Err(error) => {
                            self.message = Some(Message {
                                text: error.clone(),
                                error: true,
                            });
                            if self.setup_required {
                                self.connection = ConnectionState::NeedsServer;
                            } else {
                                self.connection = ConnectionState::Error;
                                self.connection_error = Some(error);
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
                                self.connection = ConnectionState::Error;
                                self.connection_error = Some(error);
                            }
                        }
                    }
                }
                Event::DaemonStopped(result) => {
                    self.busy = false;
                    match result {
                        Ok(()) => self.close_requested = true,
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
            self.connection = ConnectionState::Running;
            self.connection_error = None;
            self.status = Some(status);
        } else {
            self.setup_required = true;
            self.connection = ConnectionState::NeedsServer;
            self.status = Some(status);
        }
    }

    fn handle_response(&mut self, label: &str, response: Response) {
        if !response.ok {
            if label == "login"
                && response
                    .error
                    .as_ref()
                    .is_some_and(|error| error.code == error_codes::PASSWORD_CHANGE_REQUIRED)
            {
                self.login_open = false;
                self.change_password_open = true;
            }
            self.message = Some(Message {
                text: response_error(&response),
                error: true,
            });
            return;
        }

        match label {
            "server-info" => {
                self.server_info = response.data;
                self.server_info_open = true;
                self.message = Some(Message {
                    text: "server-info received".to_string(),
                    error: false,
                });
            }
            "reconnect" => {
                self.message = Some(Message {
                    text: "network state refreshed".to_string(),
                    error: false,
                });
                self.refresh_status();
            }
            "set-server" => {
                self.endpoint_open = false;
                self.message = Some(Message {
                    text: "server EndpointId saved".to_string(),
                    error: false,
                });
                self.refresh_status();
            }
            "login" => {
                self.login_open = false;
                self.login_password.clear();
                self.message = Some(Message {
                    text: "login successful".to_string(),
                    error: false,
                });
                self.refresh_status();
            }
            "change-password" => {
                self.change_password_open = false;
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

    fn handle_menu_action(&mut self, action: MenuAction) -> Option<Command> {
        match action {
            MenuAction::Refresh => {
                if self.connection == ConnectionState::Running {
                    return Some(Command::Status);
                }
            }
            MenuAction::Exit => {
                if self.worker.is_some() {
                    return Some(Command::ShutdownDaemonAndExit);
                }
                self.close_requested = true;
            }
            MenuAction::ToggleSidebar => self.show_sidebar = !self.show_sidebar,
            MenuAction::ToggleInspector => self.show_inspector = !self.show_inspector,
            MenuAction::GridView => self.grid_view = true,
            MenuAction::ListView => self.grid_view = false,
            MenuAction::ClearSearch => self.search.clear(),
            MenuAction::Login => self.login_open = true,
            MenuAction::ChangePassword => self.change_password_open = true,
            MenuAction::ServerInfo => {
                if self.connection == ConnectionState::Running {
                    return Some(Command::ServerInfo);
                }
            }
            MenuAction::ChangeEndpoint => {
                if let Some(status) = &self.status {
                    self.server_id_input = status.server_id.clone().unwrap_or_default();
                }
                self.endpoint_open = true;
            }
            MenuAction::Reconnect => {
                return Some(Command::Reconnect);
            }
            MenuAction::About => self.about_open = true,
        }
        None
    }

    fn draw_menu_bar(&mut self, ui: &mut egui::Ui) -> Option<MenuAction> {
        let mut action = None;
        let connected = self.connection == ConnectionState::Running;

        egui::Panel::top("menu_bar")
            .exact_size(30.0)
            .frame(egui::Frame::NONE.fill(theme::CHROME))
            .show(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui
                            .add_enabled(connected, egui::Button::new("Refresh"))
                            .clicked()
                        {
                            action = Some(MenuAction::Refresh);
                            ui.close();
                        }
                        ui.separator();
                        if ui.button("Exit and stop client").clicked() {
                            action = Some(MenuAction::Exit);
                            ui.close();
                        }
                    });
                    ui.menu_button("Edit", |ui| {
                        if ui
                            .add_enabled(!self.search.is_empty(), egui::Button::new("Clear Search"))
                            .clicked()
                        {
                            action = Some(MenuAction::ClearSearch);
                            ui.close();
                        }
                        ui.add_enabled(false, egui::Button::new("Settings…"));
                    });
                    ui.menu_button("View", |ui| {
                        if ui
                            .add_enabled(
                                connected,
                                egui::Button::new("Show Sidebar").selected(self.show_sidebar),
                            )
                            .clicked()
                        {
                            action = Some(MenuAction::ToggleSidebar);
                            ui.close();
                        }
                        if ui
                            .add_enabled(
                                connected,
                                egui::Button::new("Show Inspector").selected(self.show_inspector),
                            )
                            .clicked()
                        {
                            action = Some(MenuAction::ToggleInspector);
                            ui.close();
                        }
                        ui.separator();
                        if ui
                            .add_enabled(
                                connected,
                                egui::Button::new("Grid View").selected(self.grid_view),
                            )
                            .clicked()
                        {
                            action = Some(MenuAction::GridView);
                            ui.close();
                        }
                        if ui
                            .add_enabled(
                                connected,
                                egui::Button::new("List View").selected(!self.grid_view),
                            )
                            .clicked()
                        {
                            action = Some(MenuAction::ListView);
                            ui.close();
                        }
                    });
                    ui.menu_button("Server", |ui| {
                        if ui
                            .add_enabled(connected, egui::Button::new("Login…"))
                            .clicked()
                        {
                            action = Some(MenuAction::Login);
                            ui.close();
                        }
                        if ui
                            .add_enabled(connected, egui::Button::new("Change Password…"))
                            .clicked()
                        {
                            action = Some(MenuAction::ChangePassword);
                            ui.close();
                        }
                        if ui
                            .add_enabled(connected, egui::Button::new("Server Info…"))
                            .clicked()
                        {
                            action = Some(MenuAction::ServerInfo);
                            ui.close();
                        }
                        if ui
                            .add_enabled(connected, egui::Button::new("Change EndpointId…"))
                            .clicked()
                        {
                            action = Some(MenuAction::ChangeEndpoint);
                            ui.close();
                        }
                        ui.separator();
                        if ui
                            .add_enabled(!self.busy, egui::Button::new("Reconnect"))
                            .clicked()
                        {
                            action = Some(MenuAction::Reconnect);
                            ui.close();
                        }
                        if ui
                            .add_enabled(connected, egui::Button::new("Refresh Status"))
                            .clicked()
                        {
                            action = Some(MenuAction::Refresh);
                            ui.close();
                        }
                    });
                    ui.menu_button("Help", |ui| {
                        if ui.button("About aiqiezi").clicked() {
                            action = Some(MenuAction::About);
                            ui.close();
                        }
                    });
                });
            });

        action
    }

    fn draw_toolbar(&mut self, ui: &mut egui::Ui) -> Option<Command> {
        let mut pending = None;
        egui::Panel::top("toolbar")
            .exact_size(42.0)
            .frame(
                egui::Frame::NONE
                    .fill(theme::CHROME)
                    .inner_margin(egui::Margin::symmetric(10, 4))
                    .stroke(egui::Stroke::new(1.0, theme::BORDER)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Sidebar").clicked() {
                        self.show_sidebar = !self.show_sidebar;
                    }
                    if ui.button("Inspector").clicked() {
                        self.show_inspector = !self.show_inspector;
                    }
                    ui.add_enabled(false, egui::Button::new("Back"));
                    ui.add_enabled(false, egui::Button::new("Forward"));
                    ui.add_space(12.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.search)
                            .hint_text("Search photos and videos")
                            .desired_width(360.0),
                    );
                    if ui.button("Refresh").clicked() {
                        pending = Some(Command::Status);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(match self.connection {
                                ConnectionState::Running => "Connected",
                                ConnectionState::Starting => "Starting…",
                                ConnectionState::NeedsServer => "Setup required",
                                ConnectionState::Error => "Offline",
                            })
                            .color(match self.connection {
                                ConnectionState::Running => theme::SUCCESS,
                                ConnectionState::Starting => theme::CAUTION,
                                ConnectionState::NeedsServer => theme::CAUTION,
                                ConnectionState::Error => theme::ERROR,
                            }),
                        );
                    });
                });
            });
        pending
    }

    fn draw_sidebar(&mut self, ui: &mut egui::Ui) -> Option<Command> {
        let pending = None;
        egui::Frame::NONE
            .fill(theme::CHROME)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .show(ui, |ui| {
                ui.add_space(10.0);
                ui.heading("Library");
                ui.label(RichText::new("0 photos · 0 videos").color(theme::TEXT_DIM));
                ui.add_space(8.0);

                self.nav_section(
                    ui,
                    "LIBRARY",
                    &[
                        NavItem::AllPhotos,
                        NavItem::Videos,
                        NavItem::RecentlyAdded,
                        NavItem::Favorites,
                    ],
                );
                ui.add_space(8.0);
                self.nav_section(
                    ui,
                    "COLLECTIONS",
                    &[NavItem::Albums, NavItem::Folders, NavItem::Tags],
                );
                ui.add_space(8.0);
                self.nav_section(ui, "TRASH", &[NavItem::RecentlyDeleted]);

                if let Some(message) = &self.message {
                    ui.add_space(12.0);
                    ui.label(RichText::new(&message.text).color(if message.error {
                        theme::ERROR
                    } else {
                        theme::SUCCESS
                    }));
                }
            });
        pending
    }

    fn nav_section(&mut self, ui: &mut egui::Ui, title: &str, items: &[NavItem]) {
        ui.label(RichText::new(title).small().color(theme::TEXT_DIM));
        for item in items {
            let selected = self.nav == *item;
            let button = egui::Button::new(format!("  {}", item.label()))
                .frame(false)
                .selected(selected);
            if ui.add_sized([ui.available_width(), 26.0], button).clicked() {
                self.nav = *item;
            }
        }
    }

    fn draw_workspace(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.heading(self.nav.label());
            ui.label(RichText::new("0 items").color(theme::TEXT_DIM));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.selectable_label(!self.grid_view, "List").clicked() {
                    self.grid_view = false;
                }
                if ui.selectable_label(self.grid_view, "Grid").clicked() {
                    self.grid_view = true;
                }
            });
        });
        ui.add_space(6.0);
        ui.separator();

        let height = ui.available_height();
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), height),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.add_space((height * 0.22).max(40.0));
                ui.label(RichText::new("No photos or videos yet").size(20.0));
                ui.add_space(6.0);
                ui.label(
                    RichText::new("Import and upload support is coming in a later milestone.")
                        .color(theme::TEXT_DIM),
                );
                if !self.search.is_empty() {
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(format!("Search: {}", self.search)).color(theme::TEXT_DIM),
                    );
                }
            },
        );
    }

    fn draw_inspector(&mut self, ui: &mut egui::Ui) -> Option<Command> {
        let mut pending = None;
        egui::Frame::NONE
            .fill(theme::CHROME)
            .stroke(egui::Stroke::new(1.0, theme::BORDER))
            .show(ui, |ui| {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui
                        .selectable_label(self.inspector_tab == InspectorTab::Info, "Info")
                        .clicked()
                    {
                        self.inspector_tab = InspectorTab::Info;
                    }
                    if ui
                        .selectable_label(self.inspector_tab == InspectorTab::Server, "Server")
                        .clicked()
                    {
                        self.inspector_tab = InspectorTab::Server;
                    }
                });
                ui.separator();

                match self.inspector_tab {
                    InspectorTab::Info => {
                        ui.add_space(12.0);
                        ui.label(
                            RichText::new("No photo or video selected").color(theme::TEXT_DIM),
                        );
                        ui.label("Select an item to see its details.");
                    }
                    InspectorTab::Server => {
                        if let Some(status) = &self.status {
                            ui.add_space(8.0);
                            egui::Grid::new("server_info_grid")
                                .num_columns(2)
                                .spacing([10.0, 8.0])
                                .show(ui, |ui| {
                                    ui.label("Client");
                                    copyable_id(
                                        ui,
                                        &status.client_id,
                                        &short_id(&status.client_id),
                                    );
                                    ui.end_row();
                                    ui.label("Server");
                                    match &status.server_id {
                                        Some(id) => {
                                            copyable_id(ui, id, &short_id(id));
                                        }
                                        None => {
                                            ui.label(
                                                RichText::new("not configured")
                                                    .color(theme::CAUTION),
                                            );
                                        }
                                    }
                                    ui.end_row();
                                    ui.label("Socket");
                                    ui.add(
                                        egui::Label::new(RichText::new(&status.socket).monospace())
                                            .wrap(),
                                    );
                                    ui.end_row();
                                    ui.label("Version");
                                    ui.monospace(&status.version);
                                    ui.end_row();
                                });
                        }
                        ui.add_space(12.0);
                        if ui.button("Login…").clicked() {
                            self.login_open = true;
                        }
                        if ui.button("Change Password…").clicked() {
                            self.change_password_open = true;
                        }
                        if ui.button("Server Info…").clicked() {
                            pending = Some(Command::ServerInfo);
                        }
                        if ui.button("Change EndpointId…").clicked() {
                            if let Some(status) = &self.status {
                                self.server_id_input = status.server_id.clone().unwrap_or_default();
                            }
                            self.endpoint_open = true;
                        }
                        if ui.button("Reconnect").clicked() {
                            pending = Some(Command::Reconnect);
                        }
                    }
                }
                if let Some(message) = &self.message {
                    ui.add_space(10.0);
                    ui.label(RichText::new(&message.text).color(if message.error {
                        theme::ERROR
                    } else {
                        theme::SUCCESS
                    }));
                }
            });
        pending
    }

    fn draw_status_bar(&self, ui: &mut egui::Ui) {
        egui::Panel::bottom("status_bar")
            .exact_size(28.0)
            .frame(
                egui::Frame::NONE
                    .fill(theme::CHROME)
                    .inner_margin(egui::Margin::symmetric(10, 4))
                    .stroke(egui::Stroke::new(1.0, theme::BORDER)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let color = match self.connection {
                        ConnectionState::Running => theme::SUCCESS,
                        ConnectionState::Starting => theme::CAUTION,
                        ConnectionState::NeedsServer => theme::CAUTION,
                        ConnectionState::Error => theme::ERROR,
                    };
                    ui.painter().circle_filled(
                        ui.cursor().left_top() + egui::vec2(4.0, 7.0),
                        4.0,
                        color,
                    );
                    ui.add_space(12.0);
                    ui.label(match self.connection {
                        ConnectionState::Running => "Client connected",
                        ConnectionState::Starting => "Starting client daemon…",
                        ConnectionState::NeedsServer => "Server setup required",
                        ConnectionState::Error => "Client daemon unavailable",
                    });
                    ui.label(RichText::new("0 items").color(theme::TEXT_DIM));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("v{}", version::VERSION));
                        if let Some(status) = &self.status {
                            if let Some(server_id) = &status.server_id {
                                ui.label(
                                    RichText::new(short_id(server_id))
                                        .monospace()
                                        .color(theme::TEXT_DIM),
                                );
                            }
                        }
                    });
                });
            });
    }

    fn draw_setup(&mut self, ui: &mut egui::Ui) -> Option<Command> {
        let mut pending = None;
        let width = ui.available_width().min(620.0);
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), ui.available_height()),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.add_space((ui.available_height() * 0.18).max(40.0));
                ui.allocate_ui_with_layout(
                    egui::vec2(width, 0.0),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        ui.heading("Connect to aiqiezi-server");
                        ui.label(
                            "Enter the aiqiezi-server EndpointId. The GUI will start the local \
                             client daemon after saving it.",
                        );
                        ui.add_space(8.0);
                        ui.label("Get it on the server host with:");
                        ui.monospace("aiqiezi-server whoami");
                        ui.add_space(12.0);
                        ui.label("Server EndpointId");
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.server_id_input)
                                    .desired_width((width - 130.0).max(240.0))
                                    .hint_text("server EndpointId"),
                            );
                            if ui.button("Paste").clicked() {
                                match paste_endpoint_into(&mut self.server_id_input) {
                                    Ok(()) => self.message = None,
                                    Err(error) => {
                                        self.message = Some(Message {
                                            text: error,
                                            error: true,
                                        });
                                    }
                                }
                            }
                            if ui
                                .add_enabled(
                                    !self.server_id_input.is_empty(),
                                    egui::Button::new("Copy"),
                                )
                                .clicked()
                            {
                                ui.ctx().copy_text(self.server_id_input.clone());
                            }
                        });
                        ui.label(RichText::new("Ctrl+V / ⌘V also works").weak());
                        ui.add_space(8.0);
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
                        if let Some(message) = &self.message {
                            ui.add_space(10.0);
                            ui.label(RichText::new(&message.text).color(if message.error {
                                theme::ERROR
                            } else {
                                theme::SUCCESS
                            }));
                        }
                    },
                );
            },
        );
        pending
    }

    fn draw_starting(&self, ui: &mut egui::Ui) {
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), ui.available_height()),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.add_space((ui.available_height() * 0.35).max(60.0));
                ui.spinner();
                ui.add_space(8.0);
                ui.label("Starting the local client daemon…");
            },
        );
    }

    fn draw_error(&mut self, ui: &mut egui::Ui) -> Option<Command> {
        let mut pending = None;
        ui.allocate_ui_with_layout(
            egui::vec2(ui.available_width(), ui.available_height()),
            egui::Layout::top_down(egui::Align::Center),
            |ui| {
                ui.add_space((ui.available_height() * 0.3).max(50.0));
                ui.label(
                    RichText::new("Client daemon is not available")
                        .color(theme::ERROR)
                        .size(18.0),
                );
                if let Some(error) = &self.connection_error {
                    ui.label(RichText::new(error).color(theme::ERROR));
                }
                if let Some(error) = &self.config_error {
                    ui.label(RichText::new(error).color(theme::ERROR));
                }
                ui.add_space(10.0);
                if ui
                    .add_enabled(
                        self.worker.is_some() && !self.busy,
                        egui::Button::new("Retry"),
                    )
                    .clicked()
                {
                    self.connection = ConnectionState::Starting;
                    pending = Some(Command::EnsureDaemon { endpoint_id: None });
                }
                if let Some(message) = &self.message {
                    ui.add_space(8.0);
                    ui.label(RichText::new(&message.text).color(if message.error {
                        theme::ERROR
                    } else {
                        theme::SUCCESS
                    }));
                }
            },
        );
        pending
    }

    fn draw_dialogs(&mut self, ctx: &egui::Context) -> Option<Command> {
        let mut pending = None;

        if self.login_open {
            let mut open = self.login_open;
            egui::Window::new("Login")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label("Server password");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.login_password)
                            .password(true)
                            .desired_width(300.0),
                    );
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                !self.busy && !self.login_password.is_empty(),
                                egui::Button::new("Login"),
                            )
                            .clicked()
                        {
                            pending = Some(Command::Login(self.login_password.clone()));
                        }
                        if ui.button("Cancel").clicked() {
                            self.login_open = false;
                            self.login_password.clear();
                        }
                    });
                    if let Some(message) = &self.message {
                        ui.add_space(8.0);
                        ui.label(RichText::new(&message.text).color(if message.error {
                            theme::ERROR
                        } else {
                            theme::SUCCESS
                        }));
                    }
                });
            self.login_open = self.login_open && open;
        }

        if self.change_password_open {
            let mut open = self.change_password_open;
            egui::Window::new("Change Password")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .open(&mut open)
                .show(ctx, |ui| {
                    egui::Grid::new("password_dialog_grid")
                        .num_columns(2)
                        .spacing([10.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Current");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.old_password)
                                    .password(true)
                                    .desired_width(280.0),
                            );
                            ui.end_row();
                            ui.label("New");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.new_password)
                                    .password(true)
                                    .desired_width(280.0),
                            );
                            ui.end_row();
                            ui.label("Confirm");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.confirm_password)
                                    .password(true)
                                    .desired_width(280.0),
                            );
                            ui.end_row();
                        });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui
                            .add_enabled(
                                !self.busy
                                    && !self.old_password.is_empty()
                                    && !self.new_password.is_empty()
                                    && !self.confirm_password.is_empty(),
                                egui::Button::new("Change"),
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
                        if ui.button("Cancel").clicked() {
                            self.change_password_open = false;
                            self.clear_password_fields();
                        }
                    });
                    if let Some(message) = &self.message {
                        ui.add_space(8.0);
                        ui.label(RichText::new(&message.text).color(if message.error {
                            theme::ERROR
                        } else {
                            theme::SUCCESS
                        }));
                    }
                });
            self.change_password_open = self.change_password_open && open;
        }

        if self.endpoint_open {
            let mut open = self.endpoint_open;
            egui::Window::new("Change Server EndpointId")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.label("aiqiezi-server EndpointId");
                    ui.horizontal(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.server_id_input)
                                .desired_width(360.0)
                                .hint_text("server EndpointId"),
                        );
                        if ui.button("Paste").clicked() {
                            match paste_endpoint_into(&mut self.server_id_input) {
                                Ok(()) => self.message = None,
                                Err(error) => {
                                    self.message = Some(Message {
                                        text: error,
                                        error: true,
                                    });
                                }
                            }
                        }
                        if ui
                            .add_enabled(
                                !self.server_id_input.is_empty(),
                                egui::Button::new("Copy"),
                            )
                            .clicked()
                        {
                            ui.ctx().copy_text(self.server_id_input.clone());
                        }
                    });
                    ui.label(RichText::new("Ctrl+V / ⌘V also works").weak());
                    ui.add_space(8.0);
                    if ui
                        .add_enabled(
                            !self.busy && !self.server_id_input.trim().is_empty(),
                            egui::Button::new("Save"),
                        )
                        .clicked()
                    {
                        pending = Some(Command::SetServer(self.server_id_input.trim().to_string()));
                    }
                    if let Some(message) = &self.message {
                        ui.add_space(8.0);
                        ui.label(RichText::new(&message.text).color(if message.error {
                            theme::ERROR
                        } else {
                            theme::SUCCESS
                        }));
                    }
                });
            self.endpoint_open = self.endpoint_open && open;
        }

        if self.server_info_open {
            let mut open = self.server_info_open;
            let server_info_text = self
                .server_info
                .as_ref()
                .and_then(|info| serde_json::to_string_pretty(info).ok());
            egui::Window::new("Server Info")
                .collapsible(false)
                .resizable(true)
                .default_width(640.0)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .open(&mut open)
                .show(ctx, |ui| {
                    if let Some(text) = &server_info_text {
                        if ui.button("Copy JSON").clicked() {
                            ui.ctx().copy_text(text.clone());
                        }
                    }
                    egui::ScrollArea::vertical()
                        .max_height(420.0)
                        .show(ui, |ui| match &server_info_text {
                            Some(text) => {
                                ui.monospace(text);
                            }
                            None => {
                                ui.label("No server information received.");
                            }
                        });
                });
            self.server_info_open = self.server_info_open && open;
        }

        if self.about_open {
            let mut open = self.about_open;
            egui::Window::new("About aiqiezi")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .open(&mut open)
                .show(ctx, |ui| {
                    ui.heading("aiqiezi");
                    ui.label("Self-hosted photo and video archive.");
                    ui.label(format!("Version {}", version::VERSION));
                    ui.label("AGPL-3.0");
                    ui.hyperlink_to("Documentation", "https://github.com/gocpicnic/aiqiezi-gui");
                });
            self.about_open = self.about_open && open;
        }

        pending
    }

    fn clear_password_fields(&mut self) {
        self.old_password.clear();
        self.new_password.clear();
        self.confirm_password.clear();
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_events();

        let mut pending = None;
        if let Some(action) = self.draw_menu_bar(ui) {
            pending = self.handle_menu_action(action);
        }

        if self.connection == ConnectionState::Running {
            if let Some(command) = self.draw_toolbar(ui) {
                pending = Some(command);
            }
            self.draw_status_bar(ui);
            if self.show_sidebar {
                egui::Panel::left("library_sidebar")
                    .resizable(true)
                    .default_size(230.0)
                    .size_range(180.0..=360.0)
                    .frame(egui::Frame::NONE.fill(theme::CHROME))
                    .show(ui, |ui| {
                        if let Some(command) = self.draw_sidebar(ui) {
                            pending = Some(command);
                        }
                    });
            }
            if self.show_inspector {
                egui::Panel::right("inspector_sidebar")
                    .resizable(true)
                    .default_size(300.0)
                    .size_range(240.0..=440.0)
                    .frame(egui::Frame::NONE.fill(theme::CHROME))
                    .show(ui, |ui| {
                        if let Some(command) = self.draw_inspector(ui) {
                            pending = Some(command);
                        }
                    });
            }
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(theme::CANVAS))
                .show(ui, |ui| self.draw_workspace(ui));
        } else {
            egui::CentralPanel::default()
                .frame(egui::Frame::NONE.fill(theme::CANVAS))
                .show(ui, |ui| {
                    let command = match self.connection {
                        ConnectionState::NeedsServer => self.draw_setup(ui),
                        ConnectionState::Starting => {
                            self.draw_starting(ui);
                            None
                        }
                        ConnectionState::Error => self.draw_error(ui),
                        ConnectionState::Running => None,
                    };
                    if command.is_some() {
                        pending = command;
                    }
                });
        }

        if let Some(command) = self.draw_dialogs(ui.ctx()) {
            pending = Some(command);
        }

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

fn paste_endpoint_into(target: &mut String) -> Result<(), String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|error| format!("clipboard unavailable: {error}"))?;
    let text = clipboard
        .get_text()
        .map_err(|error| format!("could not read the clipboard: {error}"))?;
    let text = text.trim();
    if text.is_empty() {
        return Err("the clipboard does not contain an EndpointId".to_string());
    }
    text.parse::<EndpointId>()
        .map_err(|error| format!("clipboard does not contain a valid EndpointId: {error}"))?;
    *target = text.to_string();
    Ok(())
}

fn copyable_id(ui: &mut egui::Ui, value: &str, display: &str) {
    ui.horizontal(|ui| {
        ui.add(egui::Label::new(RichText::new(display).monospace()).wrap());
        if ui.small_button("Copy").clicked() {
            ui.ctx().copy_text(value.to_string());
        }
    });
}

fn short_id(id: &str) -> String {
    if id.len() > 20 {
        format!("{}…{}", &id[..10], &id[id.len() - 6..])
    } else {
        id.to_string()
    }
}

fn response_error(response: &Response) -> String {
    response
        .error
        .as_ref()
        .map(|error| format!("{}: {}", error.code, error.message))
        .unwrap_or_else(|| "unknown local error".to_string())
}
