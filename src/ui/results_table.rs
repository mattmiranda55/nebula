use crate::models::QueryResult;
use crate::theme::colors;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Fill, Theme};

#[derive(Debug, Clone)]
pub enum ResultsTableMessage {
    NextPage,
    PrevPage,
    ExportResults,
    CopyCell(usize, usize),
}

#[derive(Debug, Clone, Default)]
pub struct ResultsTable {
    pub result: Option<QueryResult>,
    pub page: usize,
    pub page_size: usize,
    pub error: Option<String>,
}

impl ResultsTable {
    pub fn new() -> Self {
        Self {
            result: None,
            page: 0,
            page_size: 100,
            error: None,
        }
    }

    pub fn with_result(result: QueryResult) -> Self {
        Self {
            result: Some(result),
            page: 0,
            page_size: 100,
            error: None,
        }
    }

    pub fn with_error(error: String) -> Self {
        Self {
            result: None,
            page: 0,
            page_size: 100,
            error: Some(error),
        }
    }

    pub fn view(&self) -> Element<'_, ResultsTableMessage> {
        let content: Element<'_, ResultsTableMessage> = if let Some(error) = &self.error {
            // Error display
            container(
                column![
                    text("⚠ Query Error").size(16).color(colors::DANGER),
                    Space::new().height(10),
                    text(error).size(13).color(colors::TEXT_SECONDARY),
                ]
                .padding(20),
            )
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(colors::BACKGROUND_LIGHT)),
                border: Border {
                    radius: 4.0.into(),
                    width: 1.0,
                    color: colors::DANGER,
                },
                ..Default::default()
            })
            .width(Fill)
            .into()
        } else if let Some(result) = &self.result {
            if result.columns.is_empty() && result.affected_rows.is_some() {
                // Statement result (INSERT/UPDATE/DELETE)
                container(
                    column![
                        text("✓ Query executed successfully")
                            .size(14)
                            .color(colors::SUCCESS),
                        Space::new().height(5),
                        text(format!(
                            "{} row(s) affected in {}ms",
                            result.affected_rows.unwrap_or(0),
                            result.execution_time_ms
                        ))
                        .size(13)
                        .color(colors::TEXT_SECONDARY),
                    ]
                    .padding(15),
                )
                .style(|_theme: &Theme| container::Style {
                    background: Some(Background::Color(colors::BACKGROUND_LIGHT)),
                    border: Border {
                        radius: 4.0.into(),
                        width: 1.0,
                        color: colors::SUCCESS,
                    },
                    ..Default::default()
                })
                .into()
            } else if result.rows.is_empty() {
                // Empty result
                container(
                    text("No results returned")
                        .size(14)
                        .color(colors::TEXT_MUTED),
                )
                .center_x(Fill)
                .center_y(Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(Background::Color(colors::BACKGROUND_BASE)),
                    ..Default::default()
                })
                .into()
            } else {
                // Data table
                let mut table = column![].spacing(0);

                // Header row
                let mut header_row = row![].spacing(0);
                for col in &result.columns {
                    header_row = header_row.push(
                        container(
                            text(&col.name)
                                .size(12)
                                .color(colors::TEXT_PRIMARY)
                                .font(iced::Font::MONOSPACE),
                        )
                        .padding([8, 12])
                        .width(150)
                        .style(|_theme: &Theme| container::Style {
                            background: Some(Background::Color(colors::BACKGROUND_DARKEST)),
                            border: Border {
                                radius: 0.0.into(),
                                width: 0.0,
                                color: colors::BORDER,
                            },
                            ..Default::default()
                        }),
                    );
                }
                table = table.push(header_row);

                // Data rows
                for (row_idx, data_row) in result.rows.iter().enumerate() {
                    let mut row_widget = row![].spacing(0);
                    let is_even = row_idx % 2 == 0;
                    let bg_color = if is_even {
                        colors::BACKGROUND_BASE
                    } else {
                        colors::BACKGROUND_LIGHT
                    };

                    for (_col_idx, cell) in data_row.iter().enumerate() {
                        let cell_text = cell.to_string();
                        let is_null = matches!(cell, crate::models::CellValue::Null);
                        let display_text = if is_null {
                            "NULL".to_string()
                        } else {
                            cell_text
                        };

                        row_widget = row_widget.push(
                            container(
                                text(display_text)
                                    .size(12)
                                    .color(if is_null {
                                        colors::TEXT_MUTED
                                    } else {
                                        colors::TEXT_PRIMARY
                                    })
                                    .font(iced::Font::MONOSPACE),
                            )
                            .padding([6, 12])
                            .width(150)
                            .style(move |_theme: &Theme| container::Style {
                                background: Some(Background::Color(bg_color)),
                                border: Border {
                                    radius: 0.0.into(),
                                    width: 0.0,
                                    color: colors::BORDER,
                                },
                                ..Default::default()
                            }),
                        );
                    }
                    table = table.push(row_widget);
                }

                let scrollable_table = scrollable(
                    scrollable(table).direction(scrollable::Direction::Horizontal(
                        scrollable::Scrollbar::default(),
                    )),
                )
                .height(Fill);

                // Status bar
                let status_bar = row![
                    text(format!("{} rows", result.rows.len()))
                        .size(12)
                        .color(colors::TEXT_SECONDARY),
                    Space::new().width(20),
                    text(format!("{}ms", result.execution_time_ms))
                        .size(12)
                        .color(colors::TEXT_MUTED),
                    Space::new().width(Fill),
                    button(text("Export").size(12))
                        .on_press(ResultsTableMessage::ExportResults)
                        .padding([4, 10])
                        .style(|theme: &Theme, status| {
                            button::Style {
                                background: Some(Background::Color(Color::TRANSPARENT)),
                                text_color: colors::TEXT_SECONDARY,
                                ..button::text(theme, status)
                            }
                        }),
                ]
                .align_y(Alignment::Center)
                .padding([8, 15]);

                column![
                    scrollable_table,
                    container(status_bar).style(|_theme: &Theme| {
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
                ]
                .into()
            }
        } else {
            // No query executed yet
            container(
                column![
                    text("📊").size(40),
                    Space::new().height(10),
                    text("Run a query to see results")
                        .size(14)
                        .color(colors::TEXT_MUTED),
                ]
                .align_x(Alignment::Center),
            )
            .center_x(Fill)
            .center_y(Fill)
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(colors::BACKGROUND_BASE)),
                ..Default::default()
            })
            .into()
        };

        container(content)
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(colors::BACKGROUND_BASE)),
                border: Border {
                    radius: 0.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                ..Default::default()
            })
            .width(Fill)
            .height(Fill)
            .into()
    }
}
