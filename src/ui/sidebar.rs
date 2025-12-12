use crate::db::{DatabaseInfo, TableInfo, ViewInfo};
use crate::models::ConnectionConfig;
use crate::theme::colors;
use iced::widget::{button, column, container, mouse_area, pick_list, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Fill, Length, Theme};

#[derive(Debug, Clone)]
pub enum SidebarMessage {
    SelectConnection(usize),
    ConnectionSelected(String),
    NewConnection,
    EditConnection(usize),
    DeleteConnection(usize),
    NewQuery,
    // Schema browser messages
    SelectTable(String, String),
    SelectView(String, String),
    RefreshSchema,
    ExpandDatabase(String),
    CollapseDatabase(String),
    DescribeTable(String, String),
    LoadTableData(String, String),
    // Resize messages
    StartResize,
    Resize(f32),
    EndResize,
}

#[derive(Debug, Clone, Default)]
pub struct Sidebar {
    pub connections: Vec<ConnectionConfig>,
    pub selected_connection: Option<usize>,
    pub is_connected: bool,
    pub width: f32,
    pub is_resizing: bool,
    pub min_width: f32,
    pub max_width: f32,
    // Schema browser state
    pub databases: Vec<DatabaseInfo>,
    pub tables: std::collections::HashMap<String, Vec<TableInfo>>,
    pub views: std::collections::HashMap<String, Vec<ViewInfo>>,
    pub expanded_databases: std::collections::HashSet<String>,
    pub selected_database: Option<String>,
    pub selected_table: Option<(String, String)>,
    pub is_loading: bool,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            width: 250.0,
            min_width: 150.0,
            max_width: 500.0,
            ..Default::default()
        }
    }

    fn view_header(&self) -> Element<'_, SidebarMessage> {
        container(
            row![
                text("Nebula").size(18).color(colors::PRIMARY),
                Space::new().width(Fill),
            ]
            .align_y(Alignment::Center)
            .padding([15, 15]),
        )
        .style(|_theme: &Theme| container::Style {
            background: Some(Background::Color(colors::BACKGROUND_DARKEST)),
            border: Border {
                radius: 0.0.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        })
        .into()
    }

    fn view_connection_dropdown(&self) -> Element<'_, SidebarMessage> {
        let connection_names: Vec<String> = self.connections.iter().map(|c| c.name.clone()).collect();
        let selected = self.selected_connection.and_then(|idx| {
            self.connections.get(idx).map(|c| c.name.clone())
        });

        container(
            column![
                pick_list(
                    connection_names,
                    selected,
                    SidebarMessage::ConnectionSelected,
                )
                .placeholder("Select connection...")
                .width(Fill)
                .padding([8, 12])
                .style(|theme: &Theme, status| {
                    let bg = match status {
                        pick_list::Status::Active => colors::BACKGROUND_LIGHT,
                        pick_list::Status::Hovered => colors::BACKGROUND_LIGHTER,
                        pick_list::Status::Opened { .. } => colors::BACKGROUND_LIGHTER,
                    };
                    pick_list::Style {
                        background: Background::Color(bg),
                        text_color: colors::TEXT_PRIMARY,
                        placeholder_color: colors::TEXT_MUTED,
                        handle_color: colors::TEXT_SECONDARY,
                        border: Border {
                            radius: 4.0.into(),
                            width: 1.0,
                            color: colors::BORDER,
                        },
                    }
                }),
                Space::new().height(5),
                row![
                    button(text("+").size(14).color(colors::PRIMARY))
                        .on_press(SidebarMessage::NewConnection)
                        .padding([6, 10])
                        .style(|theme: &Theme, status| {
                            let bg = match status {
                                button::Status::Hovered => colors::BACKGROUND_LIGHTER,
                                _ => colors::BACKGROUND_LIGHT,
                            };
                            button::Style {
                                background: Some(Background::Color(bg)),
                                text_color: colors::TEXT_PRIMARY,
                                border: Border {
                                    radius: 4.0.into(),
                                    width: 1.0,
                                    color: colors::BORDER,
                                },
                                ..button::text(theme, status)
                            }
                        }),
                    Space::new().width(5),
                    button(text("↻").size(14).color(colors::TEXT_SECONDARY))
                        .on_press(SidebarMessage::RefreshSchema)
                        .padding([6, 10])
                        .style(|theme: &Theme, status| {
                            let bg = match status {
                                button::Status::Hovered => colors::BACKGROUND_LIGHTER,
                                _ => colors::BACKGROUND_LIGHT,
                            };
                            button::Style {
                                background: Some(Background::Color(bg)),
                                text_color: colors::TEXT_PRIMARY,
                                border: Border {
                                    radius: 4.0.into(),
                                    width: 1.0,
                                    color: colors::BORDER,
                                },
                                ..button::text(theme, status)
                            }
                        }),
                ]
            ]
        )
        .padding([10, 10])
        .into()
    }

    fn view_schema_tree(&self) -> Element<'_, SidebarMessage> {
        let loading_indicator = if self.is_loading {
            text("⏳ Loading...").size(12).color(colors::INFO)
        } else {
            text("").size(12)
        };

        let tree_content = if self.is_loading && self.databases.is_empty() {
            column![
                Space::new().height(20),
                text("Loading databases...").size(13).color(colors::INFO),
            ]
            .align_x(Alignment::Center)
        } else if self.databases.is_empty() {
            column![
                Space::new().height(20),
                text("No databases").size(13).color(colors::TEXT_MUTED),
            ]
            .align_x(Alignment::Center)
        } else {
            let mut tree = column![].spacing(2);

            for db in &self.databases {
                let is_expanded = self.expanded_databases.contains(&db.name);
                let is_selected = self.selected_database.as_ref() == Some(&db.name);

                let db_icon = if is_expanded { "▼" } else { "▶" };
                let db_row = button(
                    row![
                        text(db_icon).size(10).color(colors::TEXT_MUTED),
                        Space::new().width(5),
                        text("🗄").size(14),
                        Space::new().width(8),
                        text(&db.name).size(13).color(if is_selected {
                            colors::PRIMARY
                        } else {
                            colors::TEXT_PRIMARY
                        }),
                    ]
                    .align_y(Alignment::Center),
                )
                .on_press(if is_expanded {
                    SidebarMessage::CollapseDatabase(db.name.clone())
                } else {
                    SidebarMessage::ExpandDatabase(db.name.clone())
                })
                .padding([6, 10])
                .width(Fill)
                .style(|theme: &Theme, status| {
                    let bg = match status {
                        button::Status::Hovered => colors::BACKGROUND_LIGHT,
                        _ => Color::TRANSPARENT,
                    };
                    button::Style {
                        background: Some(Background::Color(bg)),
                        text_color: colors::TEXT_PRIMARY,
                        border: Border::default(),
                        ..button::text(theme, status)
                    }
                });

                tree = tree.push(db_row);

                // Show tables if expanded
                if is_expanded {
                    if let Some(tables) = self.tables.get(&db.name) {
                        for table in tables {
                            let is_table_selected = self
                                .selected_table
                                .as_ref()
                                .map(|(d, t)| d == &db.name && t == &table.name)
                                .unwrap_or(false);

                            let db_name = db.name.clone();
                            let table_name = table.name.clone();
                            let db_name2 = db.name.clone();
                            let table_name2 = table.name.clone();

                            let table_row = row![
                                button(
                                    row![
                                        Space::new().width(20),
                                        text("📋").size(12),
                                        Space::new().width(8),
                                        text(&table.name).size(12).color(if is_table_selected {
                                            colors::PRIMARY
                                        } else {
                                            colors::TEXT_PRIMARY
                                        }),
                                    ]
                                    .align_y(Alignment::Center),
                                )
                                .on_press(SidebarMessage::SelectTable(
                                    db.name.clone(),
                                    table.name.clone(),
                                ))
                                .padding([4, 10])
                                .width(Fill)
                                .style(move |theme: &Theme, status| {
                                    let bg = if is_table_selected {
                                        colors::BACKGROUND_LIGHT
                                    } else {
                                        match status {
                                            button::Status::Hovered => colors::BACKGROUND_LIGHT,
                                            _ => Color::TRANSPARENT,
                                        }
                                    };
                                    button::Style {
                                        background: Some(Background::Color(bg)),
                                        text_color: colors::TEXT_PRIMARY,
                                        border: Border::default(),
                                        ..button::text(theme, status)
                                    }
                                }),
                                button(text("▶").size(10))
                                    .on_press(SidebarMessage::LoadTableData(db_name, table_name))
                                    .padding([2, 4])
                                    .style(|theme: &Theme, status| {
                                        let bg = match status {
                                            button::Status::Hovered => colors::SUCCESS,
                                            _ => Color::TRANSPARENT,
                                        };
                                        button::Style {
                                            background: Some(Background::Color(bg)),
                                            text_color: colors::TEXT_MUTED,
                                            ..button::text(theme, status)
                                        }
                                    }),
                            ]
                            .align_y(Alignment::Center);

                            tree = tree.push(table_row);
                        }
                    }

                    // Views section
                    if let Some(views) = self.views.get(&db.name) {
                        if !views.is_empty() {
                            for view in views {
                                let view_row = button(
                                    row![
                                        Space::new().width(20),
                                        text("👁").size(12),
                                        Space::new().width(8),
                                        text(&view.name).size(12).color(colors::SECONDARY),
                                    ]
                                    .align_y(Alignment::Center),
                                )
                                .on_press(SidebarMessage::SelectView(
                                    db.name.clone(),
                                    view.name.clone(),
                                ))
                                .padding([4, 10])
                                .width(Fill)
                                .style(|theme: &Theme, status| {
                                    let bg = match status {
                                        button::Status::Hovered => colors::BACKGROUND_LIGHT,
                                        _ => Color::TRANSPARENT,
                                    };
                                    button::Style {
                                        background: Some(Background::Color(bg)),
                                        text_color: colors::TEXT_PRIMARY,
                                        border: Border::default(),
                                        ..button::text(theme, status)
                                    }
                                });

                                tree = tree.push(view_row);
                            }
                        }
                    }
                }
            }

            tree
        };

        column![
            container(loading_indicator).padding([5, 10]),
            scrollable(tree_content.padding([0, 5])).height(Fill),
        ]
        .into()
    }

    fn view_connections_list(&self) -> Element<'_, SidebarMessage> {
        let new_conn_btn = button(
            row![
                text("+").size(16).color(colors::PRIMARY),
                Space::new().width(8),
                text("New Connection").size(13).color(colors::TEXT_PRIMARY),
            ]
            .align_y(Alignment::Center),
        )
        .on_press(SidebarMessage::NewConnection)
        .padding([10, 15])
        .width(Fill)
        .style(|theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered => colors::BACKGROUND_LIGHT,
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color: colors::TEXT_PRIMARY,
                border: Border {
                    radius: 4.0.into(),
                    width: 1.0,
                    color: colors::BORDER,
                },
                ..button::text(theme, status)
            }
        });

        let connections_list = if self.connections.is_empty() {
            column![
                Space::new().height(20),
                text("No connections").size(13).color(colors::TEXT_MUTED),
                Space::new().height(10),
                text("Create a new connection to get started")
                    .size(11)
                    .color(colors::TEXT_MUTED),
            ]
            .align_x(Alignment::Center)
        } else {
            let mut list = column![].spacing(4);
            for (idx, conn) in self.connections.iter().enumerate() {
                let is_selected = self.selected_connection == Some(idx);
                let icon = conn.db_type.icon();

                let conn_row = row![
                    button(
                        row![
                            text(icon).size(16),
                            Space::new().width(10),
                            column![
                                text(&conn.name).size(13).color(if is_selected {
                                    colors::PRIMARY
                                } else {
                                    colors::TEXT_PRIMARY
                                }),
                                text(format!("{}:{}", conn.host, conn.port))
                                    .size(11)
                                    .color(colors::TEXT_MUTED),
                            ]
                            .spacing(2),
                        ]
                        .align_y(Alignment::Center),
                    )
                    .on_press(SidebarMessage::SelectConnection(idx))
                    .padding([8, 12])
                    .width(Fill)
                    .style(move |theme: &Theme, status| {
                        let bg = if is_selected {
                            colors::BACKGROUND_LIGHT
                        } else {
                            match status {
                                button::Status::Hovered => colors::BACKGROUND_LIGHT,
                                _ => Color::TRANSPARENT,
                            }
                        };
                        button::Style {
                            background: Some(Background::Color(bg)),
                            text_color: colors::TEXT_PRIMARY,
                            border: if is_selected {
                                Border {
                                    radius: 4.0.into(),
                                    width: 0.0,
                                    color: Color::TRANSPARENT,
                                }
                            } else {
                                Border::default()
                            },
                            ..button::text(theme, status)
                        }
                    }),
                    button(text("✏").size(12))
                        .on_press(SidebarMessage::EditConnection(idx))
                        .padding([4, 6])
                        .style(|theme: &Theme, status| {
                            let bg = match status {
                                button::Status::Hovered => colors::BACKGROUND_LIGHTER,
                                _ => Color::TRANSPARENT,
                            };
                            button::Style {
                                background: Some(Background::Color(bg)),
                                text_color: colors::TEXT_MUTED,
                                ..button::text(theme, status)
                            }
                        }),
                    button(text("✕").size(12))
                        .on_press(SidebarMessage::DeleteConnection(idx))
                        .padding([4, 6])
                        .style(|theme: &Theme, status| {
                            let bg = match status {
                                button::Status::Hovered => colors::DANGER,
                                _ => Color::TRANSPARENT,
                            };
                            button::Style {
                                background: Some(Background::Color(bg)),
                                text_color: if matches!(status, button::Status::Hovered) {
                                    colors::TEXT_PRIMARY
                                } else {
                                    colors::TEXT_MUTED
                                },
                                ..button::text(theme, status)
                            }
                        }),
                ]
                .align_y(Alignment::Center);

                list = list.push(conn_row);
            }
            list
        };

        column![
            container(new_conn_btn).padding([10, 10]),
            container(
                column![
                    row![
                        text("Connections").size(11).color(colors::TEXT_MUTED),
                        Space::new().width(Fill),
                    ]
                    .padding([5, 12]),
                    scrollable(connections_list.padding([0, 5])).height(Fill),
                ]
            )
            .height(Fill),
        ]
        .into()
    }

    fn view_resize_handle(&self) -> Element<'_, SidebarMessage> {
        let handle_color = if self.is_resizing {
            colors::PRIMARY
        } else {
            colors::BORDER
        };

        mouse_area(
            container(Space::new().width(4).height(Fill))
                .style(move |_theme: &Theme| container::Style {
                    background: Some(Background::Color(handle_color)),
                    ..Default::default()
                })
        )
        .on_press(SidebarMessage::StartResize)
        .on_release(SidebarMessage::EndResize)
        .into()
    }

    pub fn view(&self) -> Element<'_, SidebarMessage> {
        let content = if self.is_connected {
            // Connected: show connection dropdown + schema tree
            column![
                self.view_header(),
                self.view_connection_dropdown(),
                self.view_schema_tree(),
            ]
        } else {
            // Not connected: show connections list
            column![
                self.view_header(),
                self.view_connections_list(),
            ]
        };

        let sidebar_content = container(content)
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(colors::BACKGROUND_DARK)),
                border: Border {
                    radius: 0.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            })
            .width(Length::Fixed(self.width - 4.0))
            .height(Fill);

        row![
            sidebar_content,
            self.view_resize_handle(),
        ]
        .width(Length::Fixed(self.width))
        .height(Fill)
        .into()
    }
}
