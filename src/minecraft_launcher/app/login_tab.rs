use crate::minecraft_launcher::app::{Action, Tab, TabBinding, TabTrait};
use crossterm::event::KeyCode;
use sage_auth::auth::AuthenticateBuilder;

use std::io::Stdout;

use tui::backend::CrosstermBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::Spans;
use tui::widgets::{Block, Borders, Paragraph};
use tui::Frame;
use uuid::Uuid;

pub struct LoginTab {
    account: String,
    password: String,
    uuid: Uuid,
    token: String,
    name: String,
    user_type: String,
    selected: InputBox,
    error: String,
}

impl LoginTab {
    pub fn new() -> LoginTab {
        LoginTab {
            account: "".to_string(),
            password: "".to_string(),
            uuid: Uuid::new_v4(),
            token: "".to_string(),
            name: "".to_string(),
            user_type: "".to_string(),
            selected: InputBox::Account,
            error: "".to_string(),
        }
    }

    async fn login(&mut self) {
        let request = AuthenticateBuilder::default()
            .username(self.account.clone().as_str())
            .password(self.password.clone().as_str())
            .request_user()
            .request()
            .await;

        // request.await;

        match request {
            Ok(response) => {
                self.token = response.access_token.clone();

                match response.selected_profile {
                    None => {}
                    Some(profile) => {
                        self.name = profile.name.clone();
                        self.uuid = profile.id;
                    }
                }

                self.user_type = "mojang".to_string();
                self.error = "".to_string();
            }
            Err(err) => {
                self.error = err.to_string();
            }
        };
    }
}

impl TabTrait for LoginTab {
    fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(3),
            ])
            .vertical_margin(1)
            .horizontal_margin(1)
            .split(area);

        let account_input = Paragraph::new(vec![Spans::from(self.account.clone())])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(match self.selected {
                        InputBox::Account => Color::Yellow,
                        InputBox::Password => Color::White,
                    })),
            )
            .alignment(Alignment::Center);
        f.render_widget(account_input, chunks[0]);

        let mut hidden_password = String::new();

        for _char in self.password.clone().chars() {
            hidden_password.push('*');
        }

        let password_input = Paragraph::new(vec![Spans::from(hidden_password)])
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(match self.selected {
                        InputBox::Account => Color::White,
                        InputBox::Password => Color::Yellow,
                    })),
            )
            .alignment(Alignment::Center);
        f.render_widget(password_input, chunks[2]);

        let error_text =
            Paragraph::new(vec![Spans::from(self.error.clone())]).alignment(Alignment::Center);
        f.render_widget(error_text, chunks[3])
    }

    fn on_key_press(&mut self, key_code: KeyCode) -> Action {
        match key_code {
            KeyCode::Enter => {
                let mut runtime = tokio::runtime::Runtime::new().unwrap();
                let future = self.login();
                runtime.block_on(future);

                if self.error.is_empty() {
                    Action::NextTab(Tab::Version)
                } else {
                    Action::None
                }
            }
            KeyCode::Tab => {
                match self.selected {
                    InputBox::Account => self.selected = InputBox::Password,
                    InputBox::Password => self.selected = InputBox::Account,
                }

                Action::None
            }
            KeyCode::Backspace => {
                match self.selected {
                    InputBox::Account => {
                        if !self.account.is_empty() {
                            self.account.remove(self.account.clone().len() - 1);
                        }
                    }
                    InputBox::Password => {
                        if !self.password.is_empty() {
                            self.password.remove(self.password.clone().len() - 1);
                        }
                    }
                };

                Action::None
            }
            KeyCode::Char(chr) => {
                match self.selected {
                    InputBox::Account => self.account.push(chr),
                    InputBox::Password => self.password.push(chr),
                };

                Action::None
            }
            _ => Action::None,
        }
    }

    fn get_bindings(&self) -> Vec<TabBinding> {
        let mut vec = Vec::new();

        vec.push(TabBinding::Default(
            "TAB".to_string(),
            "Select the other text input".to_string(),
        ));
        vec.push(TabBinding::Default(
            "BACKSPACE/DELETE".to_string(),
            "Remove the last character of the selected text input".to_string(),
        ));
        vec.push(TabBinding::Default(
            "ENTER".to_string(),
            "Try to login".to_string(),
        ));

        vec
    }
}

enum InputBox {
    Account,
    Password,
}
