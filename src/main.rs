mod config;
mod db;
mod models;
mod theme;
mod ui;

use config::AppConfig;
use db::{create_connection, DatabaseConnection};
use iced::{Element, Settings, Size, Task, Theme};
use models::{ConnectionState, QueryResult};
use std::sync::Arc;
use theme::{fonts, nebula_theme};
use tokio::sync::Mutex;
use ui::connection_form::ConnectionFormMessage;
use ui::main_view::{MainView, MainViewMessage, ViewState};
use ui::query_editor::QueryEditorMessage;
use ui::results_table::ResultsTableMessage;
use ui::sidebar::SidebarMessage;
use ui::tabs::TabBarMessage;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application(Nebula::new, Nebula::update, Nebula::view)
        .title("Nebula - Database Client")
        .theme(Nebula::theme)
        .subscription(Nebula::subscription)
        .settings(Settings {
            antialiasing: true,
            fonts: vec![
                fonts::IBM_PLEX_MONO_BYTES.into(),
                fonts::IBM_PLEX_MONO_BOLD_BYTES.into(),
            ],
            default_font: fonts::IBM_PLEX_MONO,
            ..Default::default()
        })
        .window_size(Size::new(1400.0, 900.0))
        .centered()
        .run()
}

struct Nebula {
    main_view: MainView,
    connection: Option<Arc<Mutex<Box<dyn DatabaseConnection>>>>,
    connection_state: ConnectionState,
    app_config: AppConfig,
    last_mouse_x: f32,
}

#[derive(Debug, Clone)]
enum Message {
    MainView(MainViewMessage),
    ConnectionResult(Result<(), String>),
    DatabasesLoaded(Result<Vec<db::DatabaseInfo>, String>),
    TablesLoaded(String, Result<Vec<db::TableInfo>, String>),
    ViewsLoaded(String, Result<Vec<db::ViewInfo>, String>),
    QueryExecuted(Result<QueryResult, String>),
    TestConnectionResult(Result<(), String>),
    TableDescribed(Result<db::TableInfo, String>),
    TableDataLoaded(Result<QueryResult, String>),
    MouseMoved(iced::Point),
    MouseReleased,
}

impl Nebula {
    fn new() -> (Self, Task<Message>) {
        // Load config from file or use default
        let app_config = AppConfig::load().unwrap_or_default();
        let saved_connections = app_config.get_connections();
        
        let mut main_view = MainView::new();
        main_view.sidebar.connections = saved_connections;
        
        (
            Self {
                main_view,
                connection: None,
                connection_state: ConnectionState::Disconnected,
                app_config,
                last_mouse_x: 0.0,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MouseMoved(position) => {
                if self.main_view.sidebar.is_resizing {
                    let delta = position.x - self.last_mouse_x;
                    let new_width = (self.main_view.sidebar.width + delta)
                        .clamp(self.main_view.sidebar.min_width, self.main_view.sidebar.max_width);
                    self.main_view.sidebar.width = new_width;
                }
                self.last_mouse_x = position.x;
                Task::none()
            }
            Message::MouseReleased => {
                self.main_view.sidebar.is_resizing = false;
                Task::none()
            }
            Message::MainView(msg) => self.handle_main_view_message(msg),
            Message::ConnectionResult(result) => {
                match result {
                    Ok(()) => {
                        self.connection_state = ConnectionState::Connected;
                        self.main_view.view_state = ViewState::Connected;
                        self.main_view.sidebar.is_connected = true;
                        self.main_view.tab_bar.add_tab("Query 1".to_string());
                        // Load databases
                        if let Some(conn) = &self.connection {
                            let conn = conn.clone();
                            return Task::perform(
                                async move {
                                    let conn = conn.lock().await;
                                    conn.list_databases()
                                        .await
                                        .map_err(|e| e.to_string())
                                },
                                Message::DatabasesLoaded,
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("Connection failed: {}", e);
                        self.connection_state = ConnectionState::Error;
                        self.main_view.connection_form.test_result = Some(Err(e));
                    }
                }
                Task::none()
            }
            Message::TestConnectionResult(result) => {
                self.main_view.connection_form.is_testing = false;
                self.main_view.connection_form.test_result = Some(result.map_err(|e| e));
                Task::none()
            }
            Message::DatabasesLoaded(result) => {
                self.main_view.sidebar.is_loading = false;
                match result {
                    Ok(databases) => {
                        self.main_view.sidebar.databases = databases;
                    }
                    Err(e) => {
                        tracing::error!("Failed to load databases: {}", e);
                    }
                }
                Task::none()
            }
            Message::TablesLoaded(database, result) => {
                match result {
                    Ok(tables) => {
                        self.main_view
                            .sidebar
                            .tables
                            .insert(database, tables);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load tables: {}", e);
                    }
                }
                Task::none()
            }
            Message::ViewsLoaded(database, result) => {
                match result {
                    Ok(views) => {
                        self.main_view.sidebar.views.insert(database, views);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load views: {}", e);
                    }
                }
                Task::none()
            }
            Message::QueryExecuted(result) => {
                match result {
                    Ok(query_result) => {
                        self.main_view.results_table =
                            ui::results_table::ResultsTable::with_result(query_result);
                    }
                    Err(e) => {
                        self.main_view.results_table =
                            ui::results_table::ResultsTable::with_error(e);
                    }
                }
                self.main_view.query_editor.is_executing = false;
                Task::none()
            }
            Message::TableDescribed(result) => {
                match result {
                    Ok(table_info) => {
                        // Display column information
                        let columns_desc: Vec<String> = table_info.columns.iter().map(|col| {
                            let pk = if col.is_primary_key { " PRIMARY KEY" } else { "" };
                            let auto = if col.is_auto_increment { " AUTO_INCREMENT" } else { "" };
                            let null = if col.nullable { " NULL" } else { " NOT NULL" };
                            let default = col.default_value.as_ref()
                                .map(|d| format!(" DEFAULT {}", d))
                                .unwrap_or_default();
                            let comment = col.comment.as_ref()
                                .map(|c| format!(" -- {}", c))
                                .unwrap_or_default();
                            format!("  {} {}{}{}{}{}{}", col.name, col.data_type, pk, auto, null, default, comment)
                        }).collect();
                        
                        self.main_view.query_editor.content = format!(
                            "-- Table: {}.{}\n-- Engine: {}\n-- Rows: {}\n-- Size: {} bytes\n\n{}",
                            table_info.database,
                            table_info.name,
                            table_info.engine.as_deref().unwrap_or("unknown"),
                            table_info.row_count.map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string()),
                            table_info.data_size.map(|s| s.to_string()).unwrap_or_else(|| "unknown".to_string()),
                            columns_desc.join("\n")
                        );
                    }
                    Err(e) => {
                        tracing::error!("Failed to describe table: {}", e);
                    }
                }
                Task::none()
            }
            Message::TableDataLoaded(result) => {
                match result {
                    Ok(query_result) => {
                        self.main_view.results_table =
                            ui::results_table::ResultsTable::with_result(query_result);
                    }
                    Err(e) => {
                        self.main_view.results_table =
                            ui::results_table::ResultsTable::with_error(e);
                    }
                }
                Task::none()
            }
        }
    }

    fn handle_main_view_message(&mut self, message: MainViewMessage) -> Task<Message> {
        match message {
            MainViewMessage::Sidebar(msg) => match msg {
                SidebarMessage::NewConnection => {
                    self.main_view.view_state = ViewState::ConnectionForm;
                    self.main_view.connection_form = ui::connection_form::ConnectionForm::new();
                    Task::none()
                }
                SidebarMessage::SelectConnection(idx) => {
                    self.main_view.sidebar.selected_connection = Some(idx);
                    Task::none()
                }
                SidebarMessage::EditConnection(idx) => {
                    if let Some(config) = self.main_view.sidebar.connections.get(idx).cloned() {
                        self.main_view.view_state = ViewState::ConnectionForm;
                        self.main_view.connection_form = ui::connection_form::ConnectionForm::with_config(config);
                    }
                    Task::none()
                }
                SidebarMessage::DeleteConnection(idx) => {
                    if idx < self.main_view.sidebar.connections.len() {
                        let conn = self.main_view.sidebar.connections.remove(idx);
                        self.app_config.remove_connection(&conn.name, conn.db_type);
                        let _ = self.app_config.save();
                    }
                    Task::none()
                }
                SidebarMessage::NewQuery => {
                    let count = self.main_view.tab_bar.tabs.len() + 1;
                    self.main_view.tab_bar.add_tab(format!("Query {}", count));
                    self.main_view.query_editor.content.clear();
                    Task::none()
                }
                SidebarMessage::ConnectionSelected(name) => {
                    // Find and select the connection by name
                    if let Some(idx) = self.main_view.sidebar.connections.iter().position(|c| c.name == name) {
                        self.main_view.sidebar.selected_connection = Some(idx);
                    }
                    Task::none()
                }
                // Schema browser messages
                SidebarMessage::ExpandDatabase(db_name) => {
                    self.main_view.sidebar.is_loading = true;
                    self.main_view
                        .sidebar
                        .expanded_databases
                        .insert(db_name.clone());
                    self.main_view.sidebar.selected_database = Some(db_name.clone());

                    if let Some(conn) = &self.connection {
                        let conn = conn.clone();
                        let conn2 = conn.clone();
                        let db = db_name.clone();
                        let db2 = db_name.clone();
                        let db3 = db_name.clone();
                        let db4 = db_name.clone();

                        let tables_task = Task::perform(
                            async move {
                                let conn = conn.lock().await;
                                conn.list_tables(&db).await.map_err(|e| e.to_string())
                            },
                            move |result| Message::TablesLoaded(db2.clone(), result),
                        );

                        let views_task = Task::perform(
                            async move {
                                let conn = conn2.lock().await;
                                conn.list_views(&db3).await.map_err(|e| e.to_string())
                            },
                            move |result| Message::ViewsLoaded(db4.clone(), result),
                        );
                        return Task::batch([tables_task, views_task]);
                    }
                    Task::none()
                }
                SidebarMessage::CollapseDatabase(db_name) => {
                    self.main_view
                        .sidebar
                        .expanded_databases
                        .remove(&db_name);
                    Task::none()
                }
                SidebarMessage::SelectTable(db, table) => {
                    self.main_view.sidebar.selected_table = Some((db.clone(), table.clone()));
                    self.main_view.query_editor.content =
                        format!("SELECT * FROM `{}`.`{}` LIMIT 100", db, table);
                    Task::none()
                }
                SidebarMessage::SelectView(db, view) => {
                    self.main_view.query_editor.content =
                        format!("SELECT * FROM `{}`.`{}` LIMIT 100", db, view);
                    Task::none()
                }
                SidebarMessage::RefreshSchema => {
                    self.main_view.sidebar.is_loading = true;
                    if let Some(conn) = &self.connection {
                        let conn = conn.clone();
                        return Task::perform(
                            async move {
                                let conn = conn.lock().await;
                                conn.list_databases().await.map_err(|e| e.to_string())
                            },
                            Message::DatabasesLoaded,
                        );
                    }
                    Task::none()
                }
                SidebarMessage::DescribeTable(db, table) => {
                    if let Some(conn) = &self.connection {
                        let conn = conn.clone();
                        return Task::perform(
                            async move {
                                let conn = conn.lock().await;
                                conn.describe_table(&db, &table).await.map_err(|e| e.to_string())
                            },
                            Message::TableDescribed,
                        );
                    }
                    Task::none()
                }
                SidebarMessage::LoadTableData(db, table) => {
                    if let Some(conn) = &self.connection {
                        let conn = conn.clone();
                        return Task::perform(
                            async move {
                                let conn = conn.lock().await;
                                conn.get_table_data(&db, &table, 100, 0).await.map_err(|e| e.to_string())
                            },
                            Message::TableDataLoaded,
                        );
                    }
                    Task::none()
                }
                SidebarMessage::StartResize => {
                    self.main_view.sidebar.is_resizing = true;
                    Task::none()
                }
                SidebarMessage::Resize(delta) => {
                    let new_width = (self.main_view.sidebar.width + delta)
                        .clamp(self.main_view.sidebar.min_width, self.main_view.sidebar.max_width);
                    self.main_view.sidebar.width = new_width;
                    Task::none()
                }
                SidebarMessage::EndResize => {
                    self.main_view.sidebar.is_resizing = false;
                    Task::none()
                }
            },
            MainViewMessage::ConnectionForm(msg) => match msg {
                ConnectionFormMessage::TestConnection => {
                    self.main_view.connection_form.is_testing = true;
                    let config = self.main_view.connection_form.config.clone();
                    Task::perform(
                        async move {
                            let conn = create_connection(&config).await.map_err(|e| e.to_string())?;
                            conn.test_connection().await.map_err(|e| e.to_string())?;
                            conn.close().await.map_err(|e| e.to_string())?;
                            Ok(())
                        },
                        Message::TestConnectionResult,
                    )
                }
                ConnectionFormMessage::SaveConnection => {
                    let config = self.main_view.connection_form.config.clone();
                    self.main_view.sidebar.connections.push(config.clone());
                    
                    // Save to config file
                    self.app_config.save_connection(&config);
                    self.app_config.set_last_connection(&config.name);
                    let _ = self.app_config.save();
                    
                    self.connection_state = ConnectionState::Connecting;

                    Task::perform(
                        async move {
                            let conn = create_connection(&config)
                                .await
                                .map_err(|e| e.to_string())?;
                            conn.test_connection().await.map_err(|e| e.to_string())?;
                            Ok::<_, String>(())
                        },
                        |result| match result {
                            Ok(_) => Message::ConnectionResult(Ok(())),
                            Err(e) => Message::ConnectionResult(Err(e)),
                        },
                    )
                }
                ConnectionFormMessage::Cancel => {
                    self.main_view.view_state = ViewState::Welcome;
                    Task::none()
                }
                other => {
                    self.main_view.connection_form.update(other);
                    Task::none()
                }
            },
            MainViewMessage::QueryEditor(msg) => match msg {
                QueryEditorMessage::QueryChanged(content) => {
                    self.main_view.query_editor.content = content;
                    Task::none()
                }
                QueryEditorMessage::ExecuteQuery => {
                    if let Some(conn) = &self.connection {
                        let conn = conn.clone();
                        let sql = self.main_view.query_editor.content.clone();
                        self.main_view.query_editor.is_executing = true;

                        // Check if it's a SELECT query or a statement
                        let is_select = sql.trim().to_uppercase().starts_with("SELECT")
                            || sql.trim().to_uppercase().starts_with("SHOW")
                            || sql.trim().to_uppercase().starts_with("DESCRIBE")
                            || sql.trim().to_uppercase().starts_with("EXPLAIN");

                        if is_select {
                            return Task::perform(
                                async move {
                                    let conn = conn.lock().await;
                                    conn.execute_query(&sql).await.map_err(|e| e.to_string())
                                },
                                Message::QueryExecuted,
                            );
                        } else {
                            return Task::perform(
                                async move {
                                    let conn = conn.lock().await;
                                    let affected = conn.execute_statement(&sql).await.map_err(|e| e.to_string())?;
                                    Ok(QueryResult {
                                        columns: vec![],
                                        rows: vec![],
                                        affected_rows: Some(affected),
                                        execution_time_ms: 0,
                                    })
                                },
                                Message::QueryExecuted,
                            );
                        }
                    }
                    Task::none()
                }
                QueryEditorMessage::ClearQuery => {
                    self.main_view.query_editor.content.clear();
                    Task::none()
                }
                QueryEditorMessage::FormatQuery => {
                    // Basic SQL formatting - add newlines after keywords
                    let formatted = self.main_view.query_editor.content
                        .replace(" FROM ", "\nFROM ")
                        .replace(" WHERE ", "\nWHERE ")
                        .replace(" AND ", "\n  AND ")
                        .replace(" OR ", "\n  OR ")
                        .replace(" ORDER BY ", "\nORDER BY ")
                        .replace(" GROUP BY ", "\nGROUP BY ")
                        .replace(" HAVING ", "\nHAVING ")
                        .replace(" JOIN ", "\nJOIN ")
                        .replace(" LEFT JOIN ", "\nLEFT JOIN ")
                        .replace(" RIGHT JOIN ", "\nRIGHT JOIN ")
                        .replace(" INNER JOIN ", "\nINNER JOIN ")
                        .replace(" LIMIT ", "\nLIMIT ");
                    self.main_view.query_editor.content = formatted;
                    Task::none()
                }
                QueryEditorMessage::SaveQuery => {
                    // Save query to history/saved queries
                    let query = models::SavedQuery::new(
                        "Saved Query".to_string(),
                        self.main_view.query_editor.content.clone(),
                    );
                    tracing::info!("Saved query: {:?}", query);
                    Task::none()
                }
            },
            MainViewMessage::Tabs(msg) => match msg {
                TabBarMessage::SelectTab(id) => {
                    self.main_view.tab_bar.active_tab = Some(id);
                    Task::none()
                }
                TabBarMessage::CloseTab(id) => {
                    self.main_view.tab_bar.close_tab(id);
                    Task::none()
                }
                TabBarMessage::NewTab => {
                    let count = self.main_view.tab_bar.tabs.len() + 1;
                    self.main_view.tab_bar.add_tab(format!("Query {}", count));
                    Task::none()
                }
            },
            MainViewMessage::Results(msg) => match msg {
                ResultsTableMessage::NextPage => {
                    self.main_view.results_table.page += 1;
                    Task::none()
                }
                ResultsTableMessage::PrevPage => {
                    if self.main_view.results_table.page > 0 {
                        self.main_view.results_table.page -= 1;
                    }
                    Task::none()
                }
                ResultsTableMessage::ExportResults => {
                    // TODO: Implement export functionality
                    tracing::info!("Export results requested");
                    Task::none()
                }
            },
        }
    }

    fn view(&self) -> Element<'_, Message> {
        self.main_view.view().map(Message::MainView)
    }

    fn theme(&self) -> Theme {
        nebula_theme()
    }

    fn subscription(&self) -> iced::Subscription<Message> {
        use iced::event::{self, Event};
        use iced::mouse;

        event::listen().map(|event| match event {
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                Message::MouseMoved(position)
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                Message::MouseReleased
            }
            _ => Message::MouseReleased, // Ignore other events
        })
    }
}

