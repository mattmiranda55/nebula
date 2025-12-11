use crate::models::ConnectionConfig;
use crate::theme::colors;
use iced::widget::{button, column, container, row, scrollable, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Fill, Theme};

#[derive(Debug, Clone)]
pub enum SidebarMessage {
    SelectConnection(usize),
    NewConnection,
    EditConnection(usize),
    DeleteConnection(usize),
    OpenQuery(usize),
    NewQuery,
}

#[derive(Debug, Clone, Default)]
pub struct Sidebar {
    pub connections: Vec<ConnectionConfig>,
    pub selected_connection: Option<usize>,
    pub is_collapsed: bool,
}

impl Sidebar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn view(&self) -> Element<'_, SidebarMessage> {
        let header = container(
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
        });

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
                text("No connections")
                    .size(13)
                    .color(colors::TEXT_MUTED),
            ]
            .align_x(Alignment::Center)
        } else {
            let mut list = column![].spacing(4);
            for (idx, conn) in self.connections.iter().enumerate() {
                let is_selected = self.selected_connection == Some(idx);
                let icon = conn.db_type.icon();

                let conn_btn = button(
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
                });

                list = list.push(conn_btn);
            }
            list
        };

        let content = column![
            header,
            container(new_conn_btn).padding([10, 10]),
            container(
                column![
                    row![
                        text("Connections")
                            .size(11)
                            .color(colors::TEXT_MUTED),
                        Space::new().width(Fill),
                    ]
                    .padding([5, 12]),
                    scrollable(connections_list.padding([0, 5])).height(Fill),
                ]
            )
            .height(Fill),
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
            .width(220)
            .height(Fill)
            .into()
    }
}
