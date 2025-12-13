mod config;
mod db;
mod models;
mod theme;

use config::AppConfig;
use db::{create_connection, DatabaseConnection, DatabaseInfo, TableInfo, ViewInfo};
use eframe::egui;
use models::{ConnectionConfig, ConnectionState, QueryResult};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Nebula - Database Client",
        options,
        Box::new(|cc| Ok(Box::new(NebulaApp::new(cc)))),
    )
}

struct NebulaApp {
    // Runtime for async operations
    runtime: tokio::runtime::Runtime,

    // Connection state
    connection: Option<Arc<Mutex<Box<dyn DatabaseConnection>>>>,
    connection_config: Option<ConnectionConfig>,
    connection_state: ConnectionState,

    // Config
    app_config: AppConfig,
    connections: Vec<ConnectionConfig>,

    // UI State
    view_state: ViewState,
    sidebar_width: f32,

    // Connection form
    form_config: ConnectionConfig,
    form_testing: bool,
    form_test_result: Option<Result<(), String>>,

    // Schema browser
    databases: Vec<DatabaseInfo>,
    tables: HashMap<String, Vec<TableInfo>>,
    views: HashMap<String, Vec<ViewInfo>>,
    expanded_databases: HashSet<String>,
    selected_database: Option<String>,
    selected_table: Option<(String, String)>,
    schema_loading: bool,

    // Query editor
    query_content: String,
    query_executing: bool,

    // Results
    query_result: Option<QueryResult>,
    result_error: Option<String>,

    // Async task results (polled each frame)
    pending_connection: Option<tokio::sync::oneshot::Receiver<Result<Box<dyn DatabaseConnection>, String>>>,
    pending_databases: Option<tokio::sync::oneshot::Receiver<Result<Vec<DatabaseInfo>, String>>>,
    pending_tables: Option<(String, tokio::sync::oneshot::Receiver<Result<Vec<TableInfo>, String>>)>,
    pending_views: Option<(String, tokio::sync::oneshot::Receiver<Result<Vec<ViewInfo>, String>>)>,
    pending_query: Option<tokio::sync::oneshot::Receiver<Result<QueryResult, String>>>,
    pending_test: Option<tokio::sync::oneshot::Receiver<Result<(), String>>>,
}

#[derive(Debug, Clone, PartialEq)]
enum ViewState {
    Welcome,
    ConnectionForm,
    Connected,
}

impl NebulaApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let app_config = AppConfig::load().unwrap_or_default();
        let connections = app_config.get_connections();

        Self {
            runtime: tokio::runtime::Runtime::new().unwrap(),
            connection: None,
            connection_config: None,
            connection_state: ConnectionState::Disconnected,
            app_config,
            connections,
            view_state: ViewState::Welcome,
            sidebar_width: 250.0,
            form_config: ConnectionConfig::default(),
            form_testing: false,
            form_test_result: None,
            databases: Vec::new(),
            tables: HashMap::new(),
            views: HashMap::new(),
            expanded_databases: HashSet::new(),
            selected_database: None,
            selected_table: None,
            schema_loading: false,
            query_content: String::new(),
            query_executing: false,
            query_result: None,
            result_error: None,
            pending_connection: None,
            pending_databases: None,
            pending_tables: None,
            pending_views: None,
            pending_query: None,
            pending_test: None,
        }
    }

    fn poll_async_tasks(&mut self) {
        // Poll connection result
        if let Some(rx) = &mut self.pending_connection {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(conn) => {
                        let conn = Arc::new(Mutex::new(conn));
                        self.connection = Some(conn.clone());
                        self.connection_state = ConnectionState::Connected;
                        self.view_state = ViewState::Connected;
                        
                        // Start loading databases
                        self.schema_loading = true;
                        let (tx, rx) = tokio::sync::oneshot::channel();
                        let conn_clone = conn.clone();
                        self.runtime.spawn(async move {
                            let conn = conn_clone.lock().await;
                            let result = conn.list_databases().await.map_err(|e| e.to_string());
                            let _ = tx.send(result);
                        });
                        self.pending_databases = Some(rx);
                    }
                    Err(e) => {
                        self.connection_state = ConnectionState::Error;
                        self.form_test_result = Some(Err(e));
                    }
                }
                self.pending_connection = None;
            }
        }

        // Poll test connection result
        if let Some(rx) = &mut self.pending_test {
            if let Ok(result) = rx.try_recv() {
                self.form_testing = false;
                self.form_test_result = Some(result);
                self.pending_test = None;
            }
        }

        // Poll databases result
        if let Some(rx) = &mut self.pending_databases {
            if let Ok(result) = rx.try_recv() {
                self.schema_loading = false;
                match result {
                    Ok(databases) => {
                        // Filter databases if specific one was configured
                        let filtered = if let Some(config) = &self.connection_config {
                            if !config.database.is_empty() {
                                databases.into_iter()
                                    .filter(|db| db.name == config.database)
                                    .collect()
                            } else {
                                databases
                            }
                        } else {
                            databases
                        };
                        
                        self.databases = filtered;
                        
                        // Auto-expand if single database
                        if self.databases.len() == 1 {
                            let db_name = self.databases[0].name.clone();
                            self.expanded_databases.insert(db_name.clone());
                            self.selected_database = Some(db_name.clone());
                            self.load_tables_and_views(&db_name);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to load databases: {}", e);
                    }
                }
                self.pending_databases = None;
            }
        }

        // Poll tables result
        if let Some((db_name, rx)) = &mut self.pending_tables {
            if let Ok(result) = rx.try_recv() {
                self.schema_loading = false;
                match result {
                    Ok(tables) => {
                        self.tables.insert(db_name.clone(), tables);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load tables: {}", e);
                    }
                }
                self.pending_tables = None;
            }
        }

        // Poll views result
        if let Some((db_name, rx)) = &mut self.pending_views {
            if let Ok(result) = rx.try_recv() {
                match result {
                    Ok(views) => {
                        self.views.insert(db_name.clone(), views);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load views: {}", e);
                    }
                }
                self.pending_views = None;
            }
        }

        // Poll query result
        if let Some(rx) = &mut self.pending_query {
            if let Ok(result) = rx.try_recv() {
                self.query_executing = false;
                match result {
                    Ok(qr) => {
                        self.query_result = Some(qr);
                        self.result_error = None;
                    }
                    Err(e) => {
                        self.query_result = None;
                        self.result_error = Some(e);
                    }
                }
                self.pending_query = None;
            }
        }
    }

    fn load_tables_and_views(&mut self, db_name: &str) {
        if let Some(conn) = &self.connection {
            self.schema_loading = true;
            
            // Load tables
            let (tx, rx) = tokio::sync::oneshot::channel();
            let conn_clone = conn.clone();
            let db = db_name.to_string();
            self.runtime.spawn(async move {
                let conn = conn_clone.lock().await;
                let result = conn.list_tables(&db).await.map_err(|e| e.to_string());
                let _ = tx.send(result);
            });
            self.pending_tables = Some((db_name.to_string(), rx));

            // Load views
            let (tx, rx) = tokio::sync::oneshot::channel();
            let conn_clone = conn.clone();
            let db = db_name.to_string();
            self.runtime.spawn(async move {
                let conn = conn_clone.lock().await;
                let result = conn.list_views(&db).await.map_err(|e| e.to_string());
                let _ = tx.send(result);
            });
            self.pending_views = Some((db_name.to_string(), rx));
        }
    }

    fn connect(&mut self) {
        let config = self.form_config.clone();
        self.connections.push(config.clone());
        self.app_config.save_connection(&config);
        let _ = self.app_config.save();
        
        self.connection_config = Some(config.clone());
        self.connection_state = ConnectionState::Connecting;

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.runtime.spawn(async move {
            let result = async {
                let conn = create_connection(&config).await.map_err(|e| e.to_string())?;
                conn.test_connection().await.map_err(|e| e.to_string())?;
                Ok(conn)
            }.await;
            let _ = tx.send(result);
        });
        self.pending_connection = Some(rx);
    }

    fn test_connection(&mut self) {
        let config = self.form_config.clone();
        self.form_testing = true;
        self.form_test_result = None;

        let (tx, rx) = tokio::sync::oneshot::channel();
        self.runtime.spawn(async move {
            let result = async {
                let conn = create_connection(&config).await.map_err(|e| e.to_string())?;
                conn.test_connection().await.map_err(|e| e.to_string())?;
                conn.close().await.map_err(|e| e.to_string())?;
                Ok(())
            }.await;
            let _ = tx.send(result);
        });
        self.pending_test = Some(rx);
    }

    fn execute_query(&mut self) {
        if let Some(conn) = &self.connection {
            let sql = self.query_content.clone();
            self.query_executing = true;

            let (tx, rx) = tokio::sync::oneshot::channel();
            let conn_clone = conn.clone();
            
            let is_select = sql.trim().to_uppercase().starts_with("SELECT")
                || sql.trim().to_uppercase().starts_with("SHOW")
                || sql.trim().to_uppercase().starts_with("DESCRIBE")
                || sql.trim().to_uppercase().starts_with("EXPLAIN");

            self.runtime.spawn(async move {
                let conn = conn_clone.lock().await;
                let result = if is_select {
                    conn.execute_query(&sql).await.map_err(|e| e.to_string())
                } else {
                    match conn.execute_statement(&sql).await {
                        Ok(affected) => Ok(QueryResult {
                            columns: vec![],
                            rows: vec![],
                            affected_rows: Some(affected),
                            execution_time_ms: 0,
                        }),
                        Err(e) => Err(e.to_string()),
                    }
                };
                let _ = tx.send(result);
            });
            self.pending_query = Some(rx);
        }
    }
}

impl eframe::App for NebulaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll async tasks
        self.poll_async_tasks();

        // Request repaint if we have pending tasks
        if self.pending_connection.is_some()
            || self.pending_databases.is_some()
            || self.pending_tables.is_some()
            || self.pending_views.is_some()
            || self.pending_query.is_some()
            || self.pending_test.is_some()
        {
            ctx.request_repaint();
        }

        // Apply dark theme
        ctx.set_visuals(theme::dark_visuals());

        match self.view_state {
            ViewState::Welcome | ViewState::ConnectionForm => {
                self.render_sidebar(ctx);
                egui::CentralPanel::default().show(ctx, |ui| {
                    if self.view_state == ViewState::ConnectionForm {
                        self.render_connection_form(ui);
                    } else {
                        self.render_welcome(ui);
                    }
                });
            }
            ViewState::Connected => {
                self.render_sidebar(ctx);
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.render_main_content(ui);
                });
            }
        }
    }
}

impl NebulaApp {
    fn render_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("sidebar")
            .resizable(true)
            .default_width(self.sidebar_width)
            .min_width(150.0)
            .max_width(500.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.heading(egui::RichText::new("Nebula").color(theme::PRIMARY));
                });
                ui.add_space(10.0);
                ui.separator();

                if self.view_state == ViewState::Connected {
                    self.render_schema_browser(ui);
                } else {
                    self.render_connections_list(ui);
                }
            });
    }

    fn render_connections_list(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        
        if ui.button("+ New Connection").clicked() {
            self.view_state = ViewState::ConnectionForm;
            self.form_config = ConnectionConfig::default();
            self.form_test_result = None;
        }

        ui.add_space(10.0);
        ui.label(egui::RichText::new("Connections").color(theme::TEXT_MUTED).small());
        ui.add_space(5.0);

        if self.connections.is_empty() {
            ui.label(egui::RichText::new("No connections").color(theme::TEXT_MUTED));
            ui.label(egui::RichText::new("Create a new connection to get started").color(theme::TEXT_MUTED).small());
        } else {
            let connections = self.connections.clone();
            for (idx, conn) in connections.iter().enumerate() {
                ui.horizontal(|ui| {
                    let btn = ui.button(format!("{} {}", conn.db_type.icon(), conn.name));
                    if btn.clicked() {
                        self.form_config = conn.clone();
                        self.view_state = ViewState::ConnectionForm;
                    }
                    
                    if ui.small_button("✕").clicked() {
                        self.connections.remove(idx);
                        self.app_config.remove_connection(&conn.name, conn.db_type);
                        let _ = self.app_config.save();
                    }
                });
                ui.label(egui::RichText::new(format!("{}:{}", conn.host, conn.port)).color(theme::TEXT_MUTED).small());
                ui.add_space(5.0);
            }
        }
    }

    fn render_schema_browser(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            if ui.button("↻ Refresh").clicked() && self.connection.is_some() {
                self.schema_loading = true;
                let conn = self.connection.as_ref().unwrap().clone();
                let (tx, rx) = tokio::sync::oneshot::channel();
                self.runtime.spawn(async move {
                    let conn = conn.lock().await;
                    let result = conn.list_databases().await.map_err(|e| e.to_string());
                    let _ = tx.send(result);
                });
                self.pending_databases = Some(rx);
            }
        });

        ui.add_space(10.0);

        if self.schema_loading && self.databases.is_empty() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Loading databases...");
            });
        } else if self.databases.is_empty() {
            ui.label(egui::RichText::new("No databases").color(theme::TEXT_MUTED));
        } else {
            // Clone data to avoid borrow issues
            let databases = self.databases.clone();
            let tables = self.tables.clone();
            let views = self.views.clone();
            let expanded = self.expanded_databases.clone();
            let selected_table = self.selected_table.clone();
            
            // Collect actions to perform after rendering
            let mut expand_db: Option<String> = None;
            let mut collapse_db: Option<String> = None;
            let mut select_table: Option<(String, String)> = None;
            let mut load_table_data: Option<(String, String)> = None;
            let mut set_query: Option<String> = None;
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                for db in &databases {
                    let is_expanded = expanded.contains(&db.name);
                    ui.horizontal(|ui| {
                        let icon = if is_expanded { "▼" } else { "▶" };
                        if ui.small_button(icon).clicked() {
                            if is_expanded {
                                collapse_db = Some(db.name.clone());
                            } else {
                                expand_db = Some(db.name.clone());
                            }
                        }
                        ui.label("🗄");
                        ui.label(&db.name);
                    });

                    if is_expanded {
                        ui.indent(&db.name, |ui| {
                            // Tables
                            if let Some(db_tables) = tables.get(&db.name) {
                                for table in db_tables {
                                    ui.horizontal(|ui| {
                                        ui.label("  📋");
                                        let selected = selected_table.as_ref()
                                            .map(|(d, t)| d == &db.name && t == &table.name)
                                            .unwrap_or(false);
                                        
                                        if ui.selectable_label(selected, &table.name).clicked() {
                                            select_table = Some((db.name.clone(), table.name.clone()));
                                            set_query = Some(format!(
                                                "SELECT * FROM `{}`.`{}` LIMIT 100",
                                                db.name, table.name
                                            ));
                                        }
                                        
                                        if ui.small_button("▶").on_hover_text("Load data").clicked() {
                                            load_table_data = Some((db.name.clone(), table.name.clone()));
                                        }
                                    });
                                }
                            }

                            // Views
                            if let Some(db_views) = views.get(&db.name) {
                                for view in db_views {
                                    ui.horizontal(|ui| {
                                        ui.label("  👁");
                                        if ui.link(&view.name).clicked() {
                                            set_query = Some(format!(
                                                "SELECT * FROM `{}`.`{}` LIMIT 100",
                                                db.name, view.name
                                            ));
                                        }
                                    });
                                }
                            }
                        });
                    }
                }
            });
            
            // Apply actions after rendering
            if let Some(db_name) = expand_db {
                self.expanded_databases.insert(db_name.clone());
                self.selected_database = Some(db_name.clone());
                if !self.tables.contains_key(&db_name) {
                    self.load_tables_and_views(&db_name);
                }
            }
            if let Some(db_name) = collapse_db {
                self.expanded_databases.remove(&db_name);
            }
            if let Some((db, table)) = select_table {
                self.selected_table = Some((db, table));
            }
            if let Some(query) = set_query {
                self.query_content = query;
            }
            if let Some((db, table)) = load_table_data {
                self.query_content = format!("SELECT * FROM `{}`.`{}` LIMIT 100", db, table);
                self.execute_query();
            }
        }
    }

    fn render_welcome(&self, ui: &mut egui::Ui) {
        ui.centered_and_justified(|ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(100.0);
                ui.heading(egui::RichText::new("Welcome to Nebula").size(32.0).color(theme::PRIMARY));
                ui.add_space(20.0);
                ui.label(egui::RichText::new("Select a connection or create a new one to get started").color(theme::TEXT_MUTED));
            });
        });
    }

    fn render_connection_form(&mut self, ui: &mut egui::Ui) {
        ui.add_space(20.0);
        ui.heading("Connection Settings");
        ui.add_space(20.0);

        egui::Grid::new("connection_form")
            .num_columns(2)
            .spacing([20.0, 10.0])
            .show(ui, |ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.form_config.name);
                ui.end_row();

                ui.label("Host:");
                ui.text_edit_singleline(&mut self.form_config.host);
                ui.end_row();

                ui.label("Port:");
                let mut port_str = self.form_config.port.to_string();
                if ui.text_edit_singleline(&mut port_str).changed() {
                    if let Ok(port) = port_str.parse() {
                        self.form_config.port = port;
                    }
                }
                ui.end_row();

                ui.label("Username:");
                ui.text_edit_singleline(&mut self.form_config.username);
                ui.end_row();

                ui.label("Password:");
                ui.add(egui::TextEdit::singleline(&mut self.form_config.password).password(true));
                ui.end_row();

                ui.label("Database:");
                ui.text_edit_singleline(&mut self.form_config.database);
                ui.end_row();
            });

        ui.add_space(20.0);

        ui.horizontal(|ui| {
            if self.form_testing {
                ui.spinner();
                ui.label("Testing connection...");
            } else {
                if ui.button("Test Connection").clicked() {
                    self.test_connection();
                }

                if ui.button("Connect").clicked() {
                    self.connect();
                }

                if ui.button("Cancel").clicked() {
                    self.view_state = ViewState::Welcome;
                }
            }
        });

        if let Some(result) = &self.form_test_result {
            ui.add_space(10.0);
            match result {
                Ok(()) => {
                    ui.label(egui::RichText::new("✓ Connection successful").color(theme::SUCCESS));
                }
                Err(e) => {
                    ui.label(egui::RichText::new(format!("✗ {}", e)).color(theme::DANGER));
                }
            }
        }
    }

    fn render_main_content(&mut self, ui: &mut egui::Ui) {
        // Query editor at top
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label("Query:");
            if self.query_executing {
                ui.spinner();
            } else {
                if ui.button("▶ Execute").clicked() {
                    self.execute_query();
                }
            }
            if ui.button("Clear").clicked() {
                self.query_content.clear();
            }
        });
        
        ui.add_space(5.0);
        
        let editor_height = 150.0;
        egui::ScrollArea::vertical()
            .max_height(editor_height)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.query_content)
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(8)
                );
            });

        ui.separator();

        // Results table
        if let Some(error) = &self.result_error {
            ui.label(egui::RichText::new(format!("Error: {}", error)).color(theme::DANGER));
        } else if let Some(result) = &self.query_result {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "{} rows × {} columns | {} ms",
                    result.rows.len(),
                    result.columns.len(),
                    result.execution_time_ms
                ));
                if let Some(affected) = result.affected_rows {
                    ui.label(format!("| {} rows affected", affected));
                }
            });
            
            ui.add_space(5.0);
            
            if !result.columns.is_empty() {
                self.render_results_table(ui, result);
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("Execute a query to see results").color(theme::TEXT_MUTED));
            });
        }
    }

    fn render_results_table(&self, ui: &mut egui::Ui, result: &QueryResult) {
        use egui_extras::{Column, TableBuilder};

        let available_height = ui.available_height();
        
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .columns(Column::auto().at_least(80.0).resizable(true), result.columns.len())
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            .header(25.0, |mut header| {
                for col in &result.columns {
                    header.col(|ui| {
                        ui.strong(&col.name);
                    });
                }
            })
            .body(|body| {
                body.rows(22.0, result.rows.len(), |mut row| {
                    let row_idx = row.index();
                    if let Some(data_row) = result.rows.get(row_idx) {
                        for cell in data_row {
                            row.col(|ui| {
                                let text = cell.display_string();
                                ui.label(&text);
                            });
                        }
                    }
                });
            });
    }
}
