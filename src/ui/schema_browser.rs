use crate::db::{DatabaseInfo, TableInfo, ViewInfo};
use crate::theme::colors;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Fill, Theme};

#[derive(Debug, Clone)]
pub enum SchemaBrowserMessage {
    SelectDatabase(String),
    SelectTable(String, String),
    SelectView(String, String),
    RefreshSchema,
    ExpandDatabase(String),
    CollapseDatabase(String),
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
        let header = row![
            text("Schema").size(14).color(colors::TEXT_PRIMARY),
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

        let tree_content = if self.databases.is_empty() {
            column![text("No databases").size(13).color(colors::TEXT_MUTED)]
                .padding([10, 15])
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

                            let table_row = button(
                                row![
                                    Space::new().width(30),
                                    text("📋").size(12),
                                    Space::new().width(8),
                                    text(&table.name).size(12).color(if is_table_selected {
                                        colors::PRIMARY
                                    } else {
                                        colors::TEXT_PRIMARY
                                    }),
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
                            });

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
                                let view_row = button(
                                    row![
                                        Space::new().width(30),
                                        text("👁").size(12),
                                        Space::new().width(8),
                                        text(&view.name).size(12).color(colors::TEXT_PRIMARY),
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
