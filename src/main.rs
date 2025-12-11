mod config;
mod db;
mod models;
mod theme;
mod ui;

use db::{create_connection, DatabaseConnection};
use iced::{Element, Font, Settings, Size, Task, Theme};
use models::{ConnectionState, QueryResult};
use std::sync::Arc;
use theme::{fonts, nebula_theme};
use tokio::sync::Mutex;
use ui::connection_form::ConnectionFormMessage;
use ui::main_view::{MainView, MainViewMessage, ViewState};
use ui::query_editor::QueryEditorMessage;
use ui::schema_browser::SchemaBrowserMessage;
use ui::sidebar::SidebarMessage;
use ui::tabs::TabBarMessage;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application(Nebula::new, Nebula::update, Nebula::view)
        .title("Nebula - Database Client")
        .theme(Nebula::theme)
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
}

impl Nebula {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                main_view: MainView::new(),
                connection: None,
                connection_state: ConnectionState::Disconnected,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MainView(msg) => self.handle_main_view_message(msg),
            Message::ConnectionResult(result) => {
                match result {
                    Ok(()) => {
                        self.connection_state = ConnectionState::Connected;
                        self.main_view.view_state = ViewState::Connected;
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
                        self.connection_state = ConnectionState::Error(e.clone());
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
                match result {
                    Ok(databases) => {
                        self.main_view.schema_browser.databases = databases;
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
                            .schema_browser
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
                        self.main_view.schema_browser.views.insert(database, views);
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
                _ => Task::none(),
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
            MainViewMessage::Schema(msg) => match msg {
                SchemaBrowserMessage::ExpandDatabase(db_name) => {
                    self.main_view
                        .schema_browser
                        .expanded_databases
                        .insert(db_name.clone());
                    self.main_view.schema_browser.selected_database = Some(db_name.clone());

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
                SchemaBrowserMessage::CollapseDatabase(db_name) => {
                    self.main_view
                        .schema_browser
                        .expanded_databases
                        .remove(&db_name);
                    Task::none()
                }
                SchemaBrowserMessage::SelectTable(db, table) => {
                    self.main_view.schema_browser.selected_table = Some((db.clone(), table.clone()));
                    self.main_view.query_editor.content =
                        format!("SELECT * FROM `{}`.`{}` LIMIT 100", db, table);
                    Task::none()
                }
                SchemaBrowserMessage::RefreshSchema => {
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
                _ => Task::none(),
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

                        return Task::perform(
                            async move {
                                let conn = conn.lock().await;
                                conn.execute_query(&sql).await.map_err(|e| e.to_string())
                            },
                            Message::QueryExecuted,
                        );
                    }
                    Task::none()
                }
                QueryEditorMessage::ClearQuery => {
                    self.main_view.query_editor.content.clear();
                    Task::none()
                }
                _ => Task::none(),
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
            MainViewMessage::Results(_msg) => {
                // Handle results messages
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        self.main_view.view().map(Message::MainView)
    }

    fn theme(&self) -> Theme {
        nebula_theme()
    }
}

