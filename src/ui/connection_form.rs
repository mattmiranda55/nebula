use crate::models::{ConnectionConfig, DatabaseType};
use crate::theme::colors;
use iced::widget::{
    button, column, container, pick_list, row, text, text_input, toggler, Space,
};
use iced::{Alignment, Background, Border, Element, Fill, Length, Theme};

#[derive(Debug, Clone)]
pub enum ConnectionFormMessage {
    NameChanged(String),
    DatabaseTypeChanged(DatabaseType),
    HostChanged(String),
    PortChanged(String),
    UsernameChanged(String),
    PasswordChanged(String),
    DatabaseChanged(String),
    SslToggled(bool),
    TestConnection,
    SaveConnection,
    Cancel,
}

pub struct ConnectionForm {
    pub config: ConnectionConfig,
    pub is_testing: bool,
    pub test_result: Option<Result<(), String>>,
}

impl Default for ConnectionForm {
    fn default() -> Self {
        Self {
            config: ConnectionConfig::default(),
            is_testing: false,
            test_result: None,
        }
    }
}

impl ConnectionForm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: ConnectionConfig) -> Self {
        Self {
            config,
            is_testing: false,
            test_result: None,
        }
    }

    pub fn update(&mut self, message: ConnectionFormMessage) {
        match message {
            ConnectionFormMessage::NameChanged(name) => {
                self.config.name = name;
            }
            ConnectionFormMessage::DatabaseTypeChanged(db_type) => {
                self.config.db_type = db_type;
                self.config.port = db_type.default_port();
            }
            ConnectionFormMessage::HostChanged(host) => {
                self.config.host = host;
            }
            ConnectionFormMessage::PortChanged(port) => {
                if let Ok(p) = port.parse() {
                    self.config.port = p;
                }
            }
            ConnectionFormMessage::UsernameChanged(username) => {
                self.config.username = username;
            }
            ConnectionFormMessage::PasswordChanged(password) => {
                self.config.password = password;
            }
            ConnectionFormMessage::DatabaseChanged(database) => {
                self.config.database = database;
            }
            ConnectionFormMessage::SslToggled(enabled) => {
                self.config.ssl_enabled = enabled;
            }
            _ => {}
        }
    }

    pub fn view(&self) -> Element<'_, ConnectionFormMessage> {
        let db_types = vec![
            DatabaseType::MySQL,
            DatabaseType::PostgreSQL,
            DatabaseType::SQLite,
            DatabaseType::MongoDB,
        ];

        let form_content = column![
            // Connection name
            text("Connection Name").size(14).color(colors::TEXT_SECONDARY),
            text_input("My Database", &self.config.name)
                .on_input(ConnectionFormMessage::NameChanged)
                .padding(10),
            Space::new().height(15),
            // Database type
            text("Database Type").size(14).color(colors::TEXT_SECONDARY),
            pick_list(
                db_types,
                Some(self.config.db_type),
                ConnectionFormMessage::DatabaseTypeChanged
            )
            .padding(10)
            .width(Fill),
            Space::new().height(15),
            // Host and Port row
            row![
                column![
                    text("Host").size(14).color(colors::TEXT_SECONDARY),
                    text_input("localhost", &self.config.host)
                        .on_input(ConnectionFormMessage::HostChanged)
                        .padding(10),
                ]
                .width(Fill),
                Space::new().width(15),
                column![
                    text("Port").size(14).color(colors::TEXT_SECONDARY),
                    text_input(
                        &self.config.db_type.default_port().to_string(),
                        &self.config.port.to_string()
                    )
                    .on_input(ConnectionFormMessage::PortChanged)
                    .padding(10)
                    .width(80),
                ]
                .width(Length::Shrink),
            ]
            .align_y(Alignment::End),
            Space::new().height(15),
            // Username and Password row
            row![
                column![
                    text("Username").size(14).color(colors::TEXT_SECONDARY),
                    text_input("root", &self.config.username)
                        .on_input(ConnectionFormMessage::UsernameChanged)
                        .padding(10),
                ]
                .width(Fill),
                Space::new().width(15),
                column![
                    text("Password").size(14).color(colors::TEXT_SECONDARY),
                    text_input("", &self.config.password)
                        .on_input(ConnectionFormMessage::PasswordChanged)
                        .padding(10)
                        .secure(true),
                ]
                .width(Fill),
            ],
            Space::new().height(15),
            // Database name
            text("Database").size(14).color(colors::TEXT_SECONDARY),
            text_input("database_name", &self.config.database)
                .on_input(ConnectionFormMessage::DatabaseChanged)
                .padding(10),
            Space::new().height(15),
            // SSL toggle
            row![
                text("Enable SSL").size(14).color(colors::TEXT_SECONDARY),
                Space::new().width(Fill),
                toggler(self.config.ssl_enabled)
                    .on_toggle(ConnectionFormMessage::SslToggled)
                    .size(20),
            ]
            .align_y(Alignment::Center),
            Space::new().height(25),
            // Test result
            if let Some(result) = &self.test_result {
                match result {
                    Ok(()) => text("✓ Connection successful!")
                        .size(14)
                        .color(colors::SUCCESS),
                    Err(e) => text(format!("✗ {}", e)).size(14).color(colors::DANGER),
                }
            } else {
                text("").size(14)
            },
            Space::new().height(10),
            // Buttons
            row![
                button(text("Test Connection").size(14))
                    .on_press(ConnectionFormMessage::TestConnection)
                    .padding([10, 20]),
                Space::new().width(Fill),
                button(text("Cancel").size(14))
                    .on_press(ConnectionFormMessage::Cancel)
                    .padding([10, 20]),
                Space::new().width(10),
                button(text("Connect").size(14))
                    .on_press(ConnectionFormMessage::SaveConnection)
                    .padding([10, 25]),
            ]
            .align_y(Alignment::Center),
        ]
        .spacing(5)
        .padding(25);

        container(form_content)
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(colors::BACKGROUND_LIGHT)),
                border: Border {
                    radius: 8.0.into(),
                    width: 1.0,
                    color: colors::BORDER,
                },
                ..Default::default()
            })
            .width(450)
            .into()
    }
}
