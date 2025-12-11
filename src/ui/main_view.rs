use crate::theme::colors;
use crate::ui::{
    connection_form::{ConnectionForm, ConnectionFormMessage},
    query_editor::{QueryEditor, QueryEditorMessage},
    results_table::{ResultsTable, ResultsTableMessage},
    schema_browser::{SchemaBrowser, SchemaBrowserMessage},
    sidebar::{Sidebar, SidebarMessage},
    tabs::{TabBar, TabBarMessage},
};
use iced::widget::{column, container, row, Space};
use iced::{Background, Element, Fill, Theme};

#[derive(Debug, Clone, Default, PartialEq)]
pub enum ViewState {
    #[default]
    Welcome,
    ConnectionForm,
    Connected,
}

#[derive(Debug, Clone)]
pub enum MainViewMessage {
    Sidebar(SidebarMessage),
    Schema(SchemaBrowserMessage),
    QueryEditor(QueryEditorMessage),
    Results(ResultsTableMessage),
    Tabs(TabBarMessage),
    ConnectionForm(ConnectionFormMessage),
}

pub struct MainView {
    pub view_state: ViewState,
    pub sidebar: Sidebar,
    pub schema_browser: SchemaBrowser,
    pub query_editor: QueryEditor,
    pub results_table: ResultsTable,
    pub tab_bar: TabBar,
    pub connection_form: ConnectionForm,
}

impl Default for MainView {
    fn default() -> Self {
        Self {
            view_state: ViewState::Welcome,
            sidebar: Sidebar::new(),
            schema_browser: SchemaBrowser::new(),
            query_editor: QueryEditor::new(),
            results_table: ResultsTable::new(),
            tab_bar: TabBar::new(),
            connection_form: ConnectionForm::new(),
        }
    }
}

impl MainView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn view(&self) -> Element<'_, MainViewMessage> {
        let sidebar = self.sidebar.view().map(MainViewMessage::Sidebar);

        let main_content: Element<'_, MainViewMessage> = match self.view_state {
            ViewState::Welcome => {
                // Welcome screen
                container(
                    column![
                        iced::widget::text("🚀").size(60),
                        Space::new().height(20),
                        iced::widget::text("Welcome to Nebula")
                            .size(28)
                            .color(colors::TEXT_PRIMARY),
                        Space::new().height(10),
                        iced::widget::text("A modern database client")
                            .size(16)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().height(30),
                        iced::widget::text("Create a new connection to get started")
                            .size(14)
                            .color(colors::TEXT_MUTED),
                    ]
                    .align_x(iced::Alignment::Center),
                )
                .center_x(Fill)
                .center_y(Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(Background::Color(colors::BACKGROUND_BASE)),
                    ..Default::default()
                })
                .into()
            }
            ViewState::ConnectionForm => {
                // Connection form
                container(
                    column![
                        Space::new().height(50),
                        iced::widget::text("New Connection")
                            .size(24)
                            .color(colors::TEXT_PRIMARY),
                        Space::new().height(30),
                        self.connection_form
                            .view()
                            .map(MainViewMessage::ConnectionForm),
                    ]
                    .align_x(iced::Alignment::Center),
                )
                .center_x(Fill)
                .center_y(Fill)
                .style(|_theme: &Theme| container::Style {
                    background: Some(Background::Color(colors::BACKGROUND_BASE)),
                    ..Default::default()
                })
                .into()
            }
            ViewState::Connected => {
                // Connected view with schema browser, query editor, and results
                let schema = self.schema_browser.view().map(MainViewMessage::Schema);
                let tabs = self.tab_bar.view().map(MainViewMessage::Tabs);
                let editor = self.query_editor.view().map(MainViewMessage::QueryEditor);
                let results = self.results_table.view().map(MainViewMessage::Results);

                row![
                    schema,
                    column![tabs, editor, results,].width(Fill).height(Fill),
                ]
                .into()
            }
        };

        row![sidebar, main_content,]
            .width(Fill)
            .height(Fill)
            .into()
    }
}
