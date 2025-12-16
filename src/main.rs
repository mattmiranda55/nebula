mod config;
mod db;
mod models;
mod theme;

use config::AppConfig;
use db::{create_connection, DatabaseConnection, DatabaseInfo, TableInfo, ViewInfo};
use eframe::egui;
use egui_phosphor_icons::icons;
use models::{BoolDisplayFormat, CellValue, ConnectionConfig, ConnectionState, QueryResult};
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
    
    // Display settings
    bool_display_format: BoolDisplayFormat,
    
    // Cell editing state
    editing_cell: Option<(usize, usize)>, // (row_idx, col_idx)
    edit_buffer: String,
    pending_edits: HashMap<(usize, usize), CellValue>, // Changed cells before committing

    // Async task results (polled each frame)
    pending_connection: Option<tokio::sync::oneshot::Receiver<Result<Box<dyn DatabaseConnection>, String>>>,
    pending_databases: Option<tokio::sync::oneshot::Receiver<Result<Vec<DatabaseInfo>, String>>>,
    pending_tables: Option<(String, tokio::sync::oneshot::Receiver<Result<Vec<TableInfo>, String>>)>,
    pending_views: Option<(String, tokio::sync::oneshot::Receiver<Result<Vec<ViewInfo>, String>>)>,
    pending_query: Option<tokio::sync::oneshot::Receiver<Result<QueryResult, String>>>,
    pending_test: Option<tokio::sync::oneshot::Receiver<Result<(), String>>>,
    pending_update: Option<tokio::sync::oneshot::Receiver<Result<u64, String>>>,
}

#[derive(Debug, Clone, PartialEq)]
enum ViewState {
    Welcome,
    ConnectionForm,
    Connected,
}

impl NebulaApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Setup Phosphor icon fonts
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor_icons::add_fonts(&mut fonts);
        cc.egui_ctx.set_fonts(fonts);

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
            bool_display_format: BoolDisplayFormat::default(),
            editing_cell: None,
            edit_buffer: String::new(),
            pending_edits: HashMap::new(),
            pending_connection: None,
            pending_databases: None,
            pending_tables: None,
            pending_views: None,
            pending_query: None,
            pending_test: None,
            pending_update: None,
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
                        // Clear any pending edits when new query results come in
                        self.pending_edits.clear();
                        self.editing_cell = None;
                    }
                    Err(e) => {
                        self.query_result = None;
                        self.result_error = Some(e);
                    }
                }
                self.pending_query = None;
            }
        }
        
        // Poll update result
        if let Some(rx) = &mut self.pending_update {
            if let Ok(result) = rx.try_recv() {
                self.query_executing = false;
                match result {
                    Ok(affected) => {
                        // Clear pending edits on success and re-run query to refresh
                        self.pending_edits.clear();
                        self.editing_cell = None;
                        tracing::info!("Update successful: {} rows affected", affected);
                        // Re-execute the query to refresh results
                        self.execute_query();
                    }
                    Err(e) => {
                        self.result_error = Some(format!("Update failed: {}", e));
                    }
                }
                self.pending_update = None;
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
            || self.pending_update.is_some()
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
        
        ui.horizontal(|ui| {
            ui.label(icons::PLUS.regular());
            if ui.button("New Connection").clicked() {
                self.view_state = ViewState::ConnectionForm;
                self.form_config = ConnectionConfig::default();
                self.form_test_result = None;
            }
        });

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
                    
                    if ui.add(egui::Button::new(icons::X.regular())).clicked() {
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
            if ui.button(icons::ARROWS_CLOCKWISE.regular()).on_hover_text("Refresh").clicked() && self.connection.is_some() {
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
                        let icon = if is_expanded { icons::CARET_DOWN } else { icons::CARET_RIGHT };
                        if ui.add(egui::Button::new(icon.regular()).frame(false)).clicked() {
                            if is_expanded {
                                collapse_db = Some(db.name.clone());
                            } else {
                                expand_db = Some(db.name.clone());
                            }
                        }
                        ui.label(icons::DATABASE.regular());
                        ui.label(&db.name);
                    });

                    if is_expanded {
                        ui.indent(&db.name, |ui| {
                            // Tables
                            if let Some(db_tables) = tables.get(&db.name) {
                                for table in db_tables {
                                    ui.horizontal(|ui| {
                                        ui.add_space(16.0);
                                        ui.label(icons::TABLE.regular());
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
                                        
                                        if ui.add(egui::Button::new(icons::PLAY.regular()).frame(false)).on_hover_text("Load data").clicked() {
                                            load_table_data = Some((db.name.clone(), table.name.clone()));
                                        }
                                    });
                                }
                            }

                            // Views
                            if let Some(db_views) = views.get(&db.name) {
                                for view in db_views {
                                    ui.horizontal(|ui| {
                                        ui.add_space(16.0);
                                        ui.label(icons::EYE.regular());
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
                    ui.horizontal(|ui| {
                        ui.label(icons::CHECK.regular().color(theme::SUCCESS));
                        ui.label(egui::RichText::new("Connection successful").color(theme::SUCCESS));
                    });
                }
                Err(e) => {
                    ui.horizontal(|ui| {
                        ui.label(icons::X.regular().color(theme::DANGER));
                        ui.label(egui::RichText::new(e).color(theme::DANGER));
                    });
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
                ui.horizontal(|ui| {
                    ui.label(icons::PLAY.regular());
                    if ui.button("Execute").clicked() {
                        self.execute_query();
                    }
                });
            }
            if ui.button("Clear").clicked() {
                self.query_content.clear();
            }
            
            ui.separator();
            
            // Boolean display format selector
            ui.label("Bool:");
            egui::ComboBox::from_id_salt("bool_format")
                .selected_text(self.bool_display_format.label())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.bool_display_format, BoolDisplayFormat::Checkbox, BoolDisplayFormat::Checkbox.label());
                    ui.selectable_value(&mut self.bool_display_format, BoolDisplayFormat::TrueFalse, BoolDisplayFormat::TrueFalse.label());
                    ui.selectable_value(&mut self.bool_display_format, BoolDisplayFormat::OneZero, BoolDisplayFormat::OneZero.label());
                });
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
        } else if let Some(result) = self.query_result.clone() {
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
                
                // Show pending edits indicator and apply button
                if !self.pending_edits.is_empty() {
                    ui.separator();
                    ui.label(egui::RichText::new(format!("{} pending edit(s)", self.pending_edits.len())).color(theme::PRIMARY));
                    
                    if ui.button(egui::RichText::new(format!("{} Apply Changes", icons::CHECK.as_str())).color(theme::SUCCESS)).clicked() {
                        self.apply_pending_edits();
                    }
                    if ui.button(egui::RichText::new(format!("{} Discard", icons::X.as_str())).color(theme::DANGER)).clicked() {
                        self.pending_edits.clear();
                        self.editing_cell = None;
                    }
                }
            });
            
            ui.add_space(5.0);
            
            if !result.columns.is_empty() {
                self.render_results_table(ui, &result);
            }
        } else {
            ui.centered_and_justified(|ui| {
                ui.label(egui::RichText::new("Execute a query to see results").color(theme::TEXT_MUTED));
            });
        }
    }

    fn render_results_table(&mut self, ui: &mut egui::Ui, result: &QueryResult) {
        use egui_extras::{Column, TableBuilder};

        let available_height = ui.available_height();
        let bool_format = self.bool_display_format;
        let editing_cell = self.editing_cell;
        let pending_edits = &self.pending_edits;
        
        // Track actions to perform after table rendering
        let mut start_edit: Option<(usize, usize, String)> = None;
        let mut commit_edit: Option<(usize, usize, String, CellValue)> = None;
        let mut cancel_edit = false;
        
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .columns(Column::auto().at_least(80.0).resizable(true), result.columns.len())
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            .sense(egui::Sense::click())
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
                        for (col_idx, cell) in data_row.iter().enumerate() {
                            row.col(|ui| {
                                let is_editing = editing_cell == Some((row_idx, col_idx));
                                let has_pending_edit = pending_edits.contains_key(&(row_idx, col_idx));
                                
                                // Get the display value (either pending edit or original)
                                let display_cell = pending_edits.get(&(row_idx, col_idx)).unwrap_or(cell);
                                
                                if is_editing {
                                    // Show nothing here - edit field is handled below
                                } else {
                                    // Display the cell value
                                    let text = display_cell.display_with_format(bool_format);
                                    let mut label = egui::RichText::new(&text);
                                    
                                    if has_pending_edit {
                                        label = label.color(theme::PRIMARY).italics();
                                    }
                                    
                                    let response = ui.add(egui::Label::new(label).sense(egui::Sense::click()));
                                    
                                    // Double-click to edit
                                    if response.double_clicked() {
                                        start_edit = Some((row_idx, col_idx, display_cell.display_string()));
                                    }
                                }
                            });
                        }
                    }
                });
            });
        
        // Handle edit field in a separate area if editing
        if let Some((row_idx, col_idx)) = self.editing_cell {
            if let Some(result) = &self.query_result {
                if let Some(data_row) = result.rows.get(row_idx) {
                    if let Some(original_cell) = data_row.get(col_idx) {
                        egui::Window::new("Edit Cell")
                            .collapsible(false)
                            .resizable(false)
                            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                            .show(ui.ctx(), |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(format!("Column: {}", result.columns.get(col_idx).map(|c| c.name.as_str()).unwrap_or("?")));
                                });
                                
                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut self.edit_buffer)
                                        .desired_width(200.0)
                                );
                                
                                // Auto-focus the text field
                                if response.gained_focus() || ui.memory(|m| m.focused().is_none()) {
                                    response.request_focus();
                                }
                                
                                ui.horizontal(|ui| {
                                    if ui.button("Save").clicked() || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                                        commit_edit = Some((row_idx, col_idx, self.edit_buffer.clone(), original_cell.clone()));
                                    }
                                    if ui.button("Cancel").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                        cancel_edit = true;
                                    }
                                });
                            });
                    }
                }
            }
        }
        
        // Apply actions
        if let Some((row_idx, col_idx, initial_value)) = start_edit {
            self.editing_cell = Some((row_idx, col_idx));
            self.edit_buffer = initial_value;
        }
        
        if let Some((row_idx, col_idx, new_value, original_cell)) = commit_edit {
            let new_cell = CellValue::parse_from_string(&new_value, &original_cell);
            self.pending_edits.insert((row_idx, col_idx), new_cell);
            self.editing_cell = None;
            self.edit_buffer.clear();
        }
        
        if cancel_edit {
            self.editing_cell = None;
            self.edit_buffer.clear();
        }
    }
    
    fn apply_pending_edits(&mut self) {
        // Need to determine the table being edited from the query
        // For now, we'll try to parse it from the query content
        let (database, table) = match self.extract_table_from_query() {
            Some((db, tbl)) => (db, tbl),
            None => {
                self.result_error = Some("Cannot determine table to update. Please use a simple SELECT query from a single table.".to_string());
                return;
            }
        };
        
        // Find the primary key column(s)
        let pk_columns: Vec<usize> = if let Some(result) = &self.query_result {
            result.columns.iter()
                .enumerate()
                .filter(|(_, col)| col.is_primary_key)
                .map(|(idx, _)| idx)
                .collect()
        } else {
            return;
        };
        
        if pk_columns.is_empty() {
            self.result_error = Some("No primary key found. Cannot safely update rows without a primary key.".to_string());
            return;
        }
        
        // Build UPDATE statements for each edited row
        let mut update_statements = Vec::new();
        let mut edited_rows: HashMap<usize, Vec<(usize, CellValue)>> = HashMap::new();
        
        // Group edits by row
        for ((row_idx, col_idx), new_value) in &self.pending_edits {
            edited_rows.entry(*row_idx).or_default().push((*col_idx, new_value.clone()));
        }
        
        if let Some(result) = &self.query_result {
            for (row_idx, edits) in edited_rows {
                if let Some(data_row) = result.rows.get(row_idx) {
                    // Build SET clause
                    let set_parts: Vec<String> = edits.iter()
                        .filter_map(|(col_idx, new_value)| {
                            result.columns.get(*col_idx).map(|col| {
                                format!("`{}` = {}", col.name, new_value.to_sql_literal())
                            })
                        })
                        .collect();
                    
                    // Build WHERE clause using primary key
                    let where_parts: Vec<String> = pk_columns.iter()
                        .filter_map(|&pk_idx| {
                            let col = result.columns.get(pk_idx)?;
                            let value = data_row.get(pk_idx)?;
                            Some(format!("`{}` = {}", col.name, value.to_sql_literal()))
                        })
                        .collect();
                    
                    if !set_parts.is_empty() && !where_parts.is_empty() {
                        let sql = format!(
                            "UPDATE `{}`.`{}` SET {} WHERE {}",
                            database,
                            table,
                            set_parts.join(", "),
                            where_parts.join(" AND ")
                        );
                        update_statements.push(sql);
                    }
                }
            }
        }
        
        if update_statements.is_empty() {
            return;
        }
        
        // Execute updates
        if let Some(conn) = &self.connection {
            self.query_executing = true;
            let (tx, rx) = tokio::sync::oneshot::channel();
            let conn_clone = conn.clone();
            
            self.runtime.spawn(async move {
                let conn = conn_clone.lock().await;
                let mut total_affected = 0u64;
                for sql in update_statements {
                    match conn.execute_statement(&sql).await {
                        Ok(affected) => total_affected += affected,
                        Err(e) => {
                            let _ = tx.send(Err(e.to_string()));
                            return;
                        }
                    }
                }
                let _ = tx.send(Ok(total_affected));
            });
            self.pending_update = Some(rx);
        }
    }
    
    fn extract_table_from_query(&self) -> Option<(String, String)> {
        // Simple parser to extract database.table or table from SELECT ... FROM ...
        let query = self.query_content.to_uppercase();
        let query_lower = &self.query_content;
        
        // Find FROM clause
        let from_idx = query.find("FROM")?;
        let after_from = &query_lower[from_idx + 4..].trim_start();
        
        // Get the table reference (until whitespace or end)
        let table_ref: String = after_from
            .chars()
            .take_while(|c| !c.is_whitespace() && *c != ';')
            .collect();
        
        // Remove backticks
        let table_ref = table_ref.replace('`', "");
        
        // Split by dot for database.table format
        let parts: Vec<&str> = table_ref.split('.').collect();
        match parts.len() {
            1 => {
                // Just table name, use selected database
                let db = self.selected_database.clone()?;
                Some((db, parts[0].to_string()))
            }
            2 => Some((parts[0].to_string(), parts[1].to_string())),
            _ => None,
        }
    }
}
