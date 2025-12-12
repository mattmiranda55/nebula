use crate::db::{DatabaseInfo, TableInfo, ViewInfo};
use crate::theme::colors;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Fill, Theme};

#[derive(Debug, Clone)]
pub enum SchemaBrowserMessage {
    SelectTable(String, String),
    SelectView(String, String),
    RefreshSchema,
    ExpandDatabase(String),
    CollapseDatabase(String),
    DescribeTable(String, String),
    LoadTableData(String, String),
}

#[derive(Debug, Clone, Default)]
pub struct SchemaBrowser {
    pub databases: Vec<DatabaseInfo>,
    pub tables: std::collections::HashMap<String, Vec<TableInfo>>,
    pub views: std::collections::HashMap<String, Vec<ViewInfo>>,
    pub expanded_databases: std::collections::HashSet<String>,
    pub selected_database: Option<String>,
    pub selected_table: Option<(String, String)>,
    pub is_loading: bool,
}

impl SchemaBrowser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn view(&self) -> Element<'_, SchemaBrowserMessage> {
        let loading_indicator = if self.is_loading {
            text("⏳").size(14).color(colors::INFO)
        } else {
            text("").size(14)
        };
        
        let header = row![
            text("Schema").size(14).color(colors::TEXT_PRIMARY),
            Space::new().width(5),
            loading_indicator,
            Space::new().width(Fill),
            button(text("↻").size(14))
                .on_press(SchemaBrowserMessage::RefreshSchema)
                .padding([4, 8])
                .style(|theme: &Theme, status| {
                    button::Style {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        text_color: colors::TEXT_SECONDARY,
                        ..button::text(theme, status)
                    }
                }),
        ]
        .align_y(Alignment::Center)
        .padding([10, 15]);

        let tree_content = if self.is_loading && self.databases.is_empty() {
            column![text("Loading...").size(13).color(colors::INFO)]
                .padding([10, 15])
        } else if self.databases.is_empty() {
            column![text("No databases").size(13).color(colors::TEXT_MUTED)]
                .padding([10, 15])
        } else {
            let mut tree = column![].spacing(2);

            for db in &self.databases {
                let is_expanded = self.expanded_databases.contains(&db.name);
                let is_selected = self.selected_database.as_ref() == Some(&db.name);

                let db_icon = if is_expanded { "▼" } else { "▶" };
                let charset_info = db.character_set.as_ref()
                    .map(|cs| format!(" ({})", cs))
                    .unwrap_or_default();
                let db_row = button(
                    row![
                        text(db_icon).size(10).color(colors::TEXT_MUTED),
                        Space::new().width(5),
                        text("🗄").size(14),
                        Space::new().width(8),
                        column![
                            text(&db.name).size(13).color(if is_selected {
                                colors::PRIMARY
                            } else {
                                colors::TEXT_PRIMARY
                            }),
                            if !charset_info.is_empty() {
                                text(charset_info).size(10).color(colors::TEXT_MUTED)
                            } else {
                                text("").size(10)
                            },
                        ],
                    ]
                    .align_y(Alignment::Center),
                )
                .on_press(if is_expanded {
                    SchemaBrowserMessage::CollapseDatabase(db.name.clone())
                } else {
                    SchemaBrowserMessage::ExpandDatabase(db.name.clone())
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
                        // Tables section
                        let tables_header = row![
                            Space::new().width(20),
                            text("Tables").size(11).color(colors::TEXT_MUTED),
                        ]
                        .padding([4, 10]);
                        tree = tree.push(tables_header);

                        for table in tables {
                            let is_table_selected = self
                                .selected_table
                                .as_ref()
                                .map(|(d, t)| d == &db.name && t == &table.name)
                                .unwrap_or(false);

                            // Format size nicely
                            let size_str = table.data_size.map(|s| {
                                if s > 1_000_000_000 {
                                    format!("{:.1}GB", s as f64 / 1_000_000_000.0)
                                } else if s > 1_000_000 {
                                    format!("{:.1}MB", s as f64 / 1_000_000.0)
                                } else if s > 1_000 {
                                    format!("{:.1}KB", s as f64 / 1_000.0)
                                } else {
                                    format!("{}B", s)
                                }
                            });

                            let engine_str = table.engine.as_ref().map(|e| e.as_str()).unwrap_or("");
                            let db_name = db.name.clone();
                            let table_name = table.name.clone();
                            let db_name2 = db.name.clone();
                            let table_name2 = table.name.clone();

                            let table_row = row![
                                button(
                                    row![
                                        Space::new().width(30),
                                        text("📋").size(12),
                                        Space::new().width(8),
                                        column![
                                            text(&table.name).size(12).color(if is_table_selected {
                                                colors::PRIMARY
                                            } else {
                                                colors::TEXT_PRIMARY
                                            }),
                                            row![
                                                text(engine_str).size(9).color(colors::TERTIARY),
                                                if let Some(size) = &size_str {
                                                    text(format!(" · {}", size)).size(9).color(colors::TEXT_MUTED)
                                                } else {
                                                    text("").size(9)
                                                },
                                            ],
                                        ],
                                        Space::new().width(Fill),
                                        if let Some(count) = table.row_count {
                                            text(format!("{}", count))
                                                .size(10)
                                                .color(colors::TEXT_MUTED)
                                        } else {
                                            text("").size(10)
                                        },
                                    ]
                                    .align_y(Alignment::Center),
                                )
                                .on_press(SchemaBrowserMessage::SelectTable(
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
                                // Describe button
                                button(text("ℹ").size(10))
                                    .on_press(SchemaBrowserMessage::DescribeTable(db_name, table_name))
                                    .padding([2, 4])
                                    .style(|theme: &Theme, status| {
                                        let bg = match status {
                                            button::Status::Hovered => colors::INFO,
                                            _ => Color::TRANSPARENT,
                                        };
                                        button::Style {
                                            background: Some(Background::Color(bg)),
                                            text_color: colors::TEXT_MUTED,
                                            ..button::text(theme, status)
                                        }
                                    }),
                                // Load data button
                                button(text("▶").size(10))
                                    .on_press(SchemaBrowserMessage::LoadTableData(db_name2, table_name2))
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
                            let views_header = row![
                                Space::new().width(20),
                                text("Views").size(11).color(colors::TEXT_MUTED),
                            ]
                            .padding([4, 10]);
                            tree = tree.push(views_header);

                            for view in views {
                                let has_definition = view.definition.is_some();
                                let view_row = button(
                                    row![
                                        Space::new().width(30),
                                        text("👁").size(12),
                                        Space::new().width(8),
                                        column![
                                            text(&view.name).size(12).color(colors::SECONDARY),
                                            if has_definition {
                                                text("(view)").size(9).color(colors::TEXT_MUTED)
                                            } else {
                                                text("").size(9)
                                            },
                                        ],
                                    ]
                                    .align_y(Alignment::Center),
                                )
                                .on_press(SchemaBrowserMessage::SelectView(
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

        let content = column![
            container(header).style(|_theme: &Theme| {
                container::Style {
                    background: Some(Background::Color(colors::BACKGROUND_DARKEST)),
                    border: Border {
                        radius: 0.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..Default::default()
                }
            }),
            scrollable(tree_content).height(Fill),
        ];

        container(content)
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(colors::BACKGROUND_DARK)),
                border: Border {
                    radius: 0.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            })
            .width(250)
            .height(Fill)
            .into()
    }
}
