use crate::theme::colors;
use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Background, Border, Color, Element, Fill, Theme};

#[derive(Debug, Clone)]
pub enum QueryEditorMessage {
    QueryChanged(String),
    ExecuteQuery,
    FormatQuery,
    ClearQuery,
    SaveQuery,
}

#[derive(Debug, Clone, Default)]
pub struct QueryEditor {
    pub content: String,
    pub is_executing: bool,
}

impl QueryEditor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_content(content: String) -> Self {
        Self {
            content,
            is_executing: false,
        }
    }

    pub fn view(&self) -> Element<'_, QueryEditorMessage> {
        let toolbar = row![
            button(
                row![text("▶").size(12), Space::new().width(6), text("Run").size(13),]
                    .align_y(Alignment::Center)
            )
            .on_press(QueryEditorMessage::ExecuteQuery)
            .padding([8, 16])
            .style(|theme: &Theme, status| {
                let bg = match status {
                    button::Status::Hovered => colors::PRIMARY_LIGHT,
                    button::Status::Pressed => colors::PRIMARY_DARK,
                    _ => colors::PRIMARY,
                };
                button::Style {
                    background: Some(Background::Color(bg)),
                    text_color: colors::BACKGROUND_DARKEST,
                    border: Border {
                        radius: 4.0.into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                    ..button::primary(theme, status)
                }
            }),
            Space::new().width(10),
            button(text("Format").size(13))
                .on_press(QueryEditorMessage::FormatQuery)
                .padding([8, 12])
                .style(|theme: &Theme, status| {
                    button::Style {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        text_color: colors::TEXT_SECONDARY,
                        ..button::text(theme, status)
                    }
                }),
            button(text("Clear").size(13))
                .on_press(QueryEditorMessage::ClearQuery)
                .padding([8, 12])
                .style(|theme: &Theme, status| {
                    button::Style {
                        background: Some(Background::Color(Color::TRANSPARENT)),
                        text_color: colors::TEXT_SECONDARY,
                        ..button::text(theme, status)
                    }
                }),
            Space::new().width(Fill),
            button(text("Save").size(13))
                .on_press(QueryEditorMessage::SaveQuery)
                .padding([8, 12])
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

        let editor_input = text_input("SELECT * FROM table_name", &self.content)
            .on_input(QueryEditorMessage::QueryChanged)
            .padding(15)
            .size(14)
            .style(|_theme: &Theme, _status| text_input::Style {
                background: Background::Color(colors::BACKGROUND_DARK),
                border: Border {
                    radius: 0.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                icon: colors::TEXT_MUTED,
                placeholder: colors::TEXT_MUTED,
                value: colors::TEXT_PRIMARY,
                selection: colors::PRIMARY,
            });

        let content = column![
            container(toolbar).style(|_theme: &Theme| {
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
            container(editor_input)
                .style(|_theme: &Theme| {
                    container::Style {
                        background: Some(Background::Color(colors::BACKGROUND_DARK)),
                        border: Border {
                            radius: 0.0.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        ..Default::default()
                    }
                })
                .width(Fill)
                .height(200),
        ];

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
            .into()
    }
}
