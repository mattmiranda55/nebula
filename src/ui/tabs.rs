use crate::theme::colors;
use iced::widget::{button, container, row, text, Space};
use iced::{Alignment, Background, Border, Color, Element, Fill, Theme};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Tab {
    pub id: Uuid,
    pub title: String,
    pub is_modified: bool,
}

impl Tab {
    pub fn new(title: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            is_modified: false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TabBarMessage {
    SelectTab(Uuid),
    CloseTab(Uuid),
    NewTab,
}

#[derive(Debug, Clone, Default)]
pub struct TabBar {
    pub tabs: Vec<Tab>,
    pub active_tab: Option<Uuid>,
}

impl TabBar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_tab(&mut self, title: String) -> Uuid {
        let tab = Tab::new(title);
        let id = tab.id;
        self.tabs.push(tab);
        self.active_tab = Some(id);
        id
    }

    pub fn close_tab(&mut self, id: Uuid) {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == id) {
            self.tabs.remove(pos);
            if self.active_tab == Some(id) {
                self.active_tab = self.tabs.get(pos.saturating_sub(1)).map(|t| t.id);
            }
        }
    }

    pub fn view(&self) -> Element<'_, TabBarMessage> {
        let mut tabs_row = row![].spacing(0);

        for tab in &self.tabs {
            let is_active = self.active_tab == Some(tab.id);
            let tab_id = tab.id;

            let tab_content = row![
                text(&tab.title).size(12).color(if is_active {
                    colors::TEXT_PRIMARY
                } else {
                    colors::TEXT_SECONDARY
                }),
                if tab.is_modified {
                    text(" •").size(12).color(colors::PRIMARY)
                } else {
                    text("").size(12)
                },
                Space::new().width(8),
                button(text("×").size(12))
                    .on_press(TabBarMessage::CloseTab(tab_id))
                    .padding([2, 6])
                    .style(|theme: &Theme, status| {
                        let text_color = match status {
                            button::Status::Hovered => colors::DANGER,
                            _ => colors::TEXT_MUTED,
                        };
                        button::Style {
                            background: Some(Background::Color(Color::TRANSPARENT)),
                            text_color,
                            ..button::text(theme, status)
                        }
                    }),
            ]
            .align_y(Alignment::Center);

            let tab_btn = button(tab_content)
                .on_press(TabBarMessage::SelectTab(tab_id))
                .padding([8, 12])
                .style(move |theme: &Theme, status| {
                    let bg = if is_active {
                        colors::BACKGROUND_BASE
                    } else {
                        match status {
                            button::Status::Hovered => colors::BACKGROUND_LIGHT,
                            _ => colors::BACKGROUND_DARK,
                        }
                    };
                    button::Style {
                        background: Some(Background::Color(bg)),
                        text_color: colors::TEXT_PRIMARY,
                        border: Border {
                            radius: 0.0.into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        ..button::text(theme, status)
                    }
                });

            tabs_row = tabs_row.push(tab_btn);
        }

        // New tab button
        let new_tab_btn = button(text("+").size(14))
            .on_press(TabBarMessage::NewTab)
            .padding([8, 12])
            .style(|theme: &Theme, status| {
                let bg = match status {
                    button::Status::Hovered => colors::BACKGROUND_LIGHT,
                    _ => Color::TRANSPARENT,
                };
                button::Style {
                    background: Some(Background::Color(bg)),
                    text_color: colors::TEXT_MUTED,
                    ..button::text(theme, status)
                }
            });

        tabs_row = tabs_row.push(new_tab_btn);
        tabs_row = tabs_row.push(Space::new().width(Fill));

        container(tabs_row)
            .style(|_theme: &Theme| container::Style {
                background: Some(Background::Color(colors::BACKGROUND_DARK)),
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
