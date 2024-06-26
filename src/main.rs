#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use iced::{
    advanced::Widget,
    keyboard::{self, Key},
    theme,
    widget::{button, column, container, row, space::Space, text, text_input},
    Alignment, Application, Command, Element, Length, Pixels, Size, Theme,
};
use iced_aw::menu::{self, Item};
use iced_aw::{grid, grid_row};
use rand::{seq::SliceRandom, thread_rng};
use std::{path::PathBuf, sync::Arc, vec};

use grammar::*;
mod settings;
mod style;

fn main() -> iced::Result {
    App::run(iced::Settings {
        window: iced::window::Settings {
            size: Size::new(600., 235.),
            min_size: Some(Size::new(600., 235.)),
            ..Default::default()
        },
        ..Default::default()
    })
}

#[derive(Debug)]
struct App {
    debug_layout: bool,
    content: Vec<Entry>,
    current: Option<usize>,
    entry: String,
    error: Option<Error>,
    file: Option<PathBuf>,
    langs: [Lang; 2],
    state: State,
    last_score: f32,
    dark_theme: bool,
    total_score: (f32, usize),
    font_size: Pixels,
    spacing: f32,
}

impl App {
    fn init(&mut self, mut content: Vec<Entry>) {
        self.entry = String::new();
        self.current = Some(0);
        content.shuffle(&mut thread_rng());
        self.content = content;
        self.total_score = (0., self.content.len());
        self.last_score = 0.;
        self.state = State::WaitUserAnswer;
    }
    fn correct(&mut self) {
        self.last_score = self.content[self.current.unwrap()].correct(
            &self.entry.trim().into(),
            0,
            &self.langs[0],
        );
        self.total_score.0 += self.last_score;
        self.state = State::Correcting;
    }
    fn next(&mut self) {
        self.entry = String::new();
        match self.current {
            Some(nb) => {
                self.current = if nb + 1 == self.content.len() {
                    self.state = State::End;
                    None
                } else {
                    self.state = State::WaitUserAnswer;
                    Some(nb + 1)
                }
            }
            None => (),
        }
    }
}

impl Default for App {
    fn default() -> Self {
        let mut default_content = vec![
            Entry("yes".into(), "oui".into(), GramClass::Adverb),
            Entry("no".into(), "non".into(), GramClass::Adverb),
            Entry("the work".into(), "le travail".into(), GramClass::Noun),
            Entry("the rust".into(), "la rouille".into(), GramClass::Noun),
            Entry("the solution".into(), "la solution".into(), GramClass::Noun),
            Entry("to rise".into(), "s'élever".into(), GramClass::Verb),
        ];
        default_content.shuffle(&mut thread_rng());
        Self {
            debug_layout: false,
            total_score: (0., default_content.len()),
            content: default_content,
            current: Some(0),
            entry: String::new(),
            error: None,
            file: None,
            langs: ["English".into(), "French".into()],
            state: State::WaitUserAnswer,
            last_score: 0.,
            dark_theme: true,
            font_size: Pixels(16.),
            spacing: 5.0,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    DebugToggle,
    TextInputChanged(String),
    OpenFile,
    FileOpened(Result<(PathBuf, Arc<([Lang; 2], Vec<Entry>)>), Error>),
    Correction,
    Next,
    // None,
    Start,
    Enter,
    ThemeSelected,
    TextFontChanged(f32),
    SpacingChanged(f32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Error {
    IoError,
    DialogClosed,
    ParseError,
}

#[derive(Debug)]
enum State {
    Correcting,
    WaitUserAnswer,
    End,
}

impl iced::Application for App {
    type Message = Message;
    type Theme = Theme;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flag: Self::Flags) -> (Self, Command<Message>) {
        (Self::default(), Command::none())
    }

    fn title(&self) -> String {
        match &self.file {
            Some(path) => format!("{} — ULang ", path.to_str().unwrap_or("")),
            None => String::from("ULang"),
        }
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        keyboard::on_key_press(|key, modifiers| match key.as_ref() {
            Key::Character("o") if modifiers.command() => Some(Message::OpenFile), // Ctrl + o
            Key::Named(keyboard::key::Named::Enter) => Some(Message::Enter),       // Enter
            _ => None,
        })
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::DebugToggle => {
                self.debug_layout = !self.debug_layout;
                Command::none()
            }
            Message::TextInputChanged(value) => {
                self.entry = value;
                Command::none()
            }
            Message::OpenFile => Command::perform(pick_file(), Message::FileOpened),
            Message::FileOpened(result) => {
                match result {
                    Ok((path, content)) => {
                        self.langs = content.0.clone();
                        self.init(content.1.clone());
                        self.file = Some(path);
                        self.error = None;
                    }
                    Err(Error::DialogClosed) => (),
                    Err(err) => self.error = Some(err),
                }
                Command::none()
            }
            Message::Enter => {
                match self.state {
                    State::WaitUserAnswer => self.correct(),
                    State::Correcting => self.next(),
                    _ => (),
                }
                Command::none()
            }
            Message::Correction => {
                self.correct();
                Command::none()
            }
            Message::Next => {
                self.next();
                Command::none()
            }
            // Message::None => Command::none(),
            Message::Start => {
                if let Some(_) = self.file {
                    self.init(self.content.clone());
                } else {
                    self.init(App::default().content);
                }
                self.state = State::WaitUserAnswer;
                Command::none()
            }
            Message::ThemeSelected => {
                self.dark_theme = !self.dark_theme;
                Command::none()
            }
            Message::TextFontChanged(new_size) => {
                self.font_size.0 = new_size;
                Command::none()
            }
            Message::SpacingChanged(new_spacing) => {
                self.spacing = new_spacing;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // Declarating widgets
        let lang_one = text(&self.langs[0]).size(self.font_size);
        let lang_two = text(&self.langs[1]).size(self.font_size);

        let known = text(match self.current {
            Some(nb) => self.content[nb].get(1),
            None => "".into(),
        })
        .size(self.font_size);

        let next_button = button(
            text(match self.state {
                State::Correcting => "Next",
                State::WaitUserAnswer => "Correct",
                State::End => "Restart",
                // _ => "",
            })
            .size(self.font_size)
            .width(self.font_size * 4.0),
        )
        .on_press(match self.state {
            State::Correcting => Message::Next,
            State::WaitUserAnswer => Message::Correction,
            State::End => Message::Start,
            // _ => Message::None,
        });

        let open = button(text("Open").size(self.font_size))
            .on_press(Message::OpenFile)
            .style(theme::Button::Custom(Box::new(style::Header::from(
                &self.theme(),
            ))));

        let menu_tpl = |items| {
            menu::Menu::new(items)
                .max_width(180.0)
                .offset(5.0)
                .spacing(5.0)
        };

        let error_log = text(match &self.error {
            Some(err) => format!("{:?}: invalid file", err),
            None => "".to_string(),
        })
        .size(self.font_size);
        // End declatating widgets

        // Grouping widgets

        // Header
        #[rustfmt::skip]
        let header = iced_aw::menu_bar!(
            (button(text("File").size(self.font_size))
                .style(theme::Button::Custom(Box::new(style::Header::from(&self.theme())))), // see
                // in src/style.rs
            {
                let size = Widget::size(&open).width;
                menu_tpl(iced_aw::menu_items!((open))).width(size)
            })
            (button(text("Settings").size(self.font_size))
                .style(theme::Button::Custom(Box::new(style::Header::from(&self.theme())))),
            {
                self.view_settings() // see in src/settings.rs
            })
        ).padding(3.0);

        // Main
        let mut variable = row![].width(self.font_size.0 * 20.0);
        match self.state {
            State::WaitUserAnswer => {
                variable = variable.push(
                    text_input("Write your answer", &self.entry)
                        .size(self.font_size)
                        .line_height(iced::widget::text::LineHeight::Relative(1.0))
                        .on_input(Message::TextInputChanged)
                        .on_submit(Message::Correction),
                );
            }
            State::Correcting => {
                variable = variable.push(text(&self.entry).size(self.font_size));
                if self.current.is_some() && !self.entry.is_empty() {
                    variable = variable.push(Space::new(10, 0));
                }
                if let Some(nb) = &self.current {
                    variable = variable.push(text(&self.content[*nb].get(0)).size(self.font_size))
                }
            }
            _ => (),
        }

        let main = grid!(grid_row!(lang_one, variable), grid_row!(lang_two, known))
            .spacing(self.spacing)
            .padding(self.spacing);

        // Score
        let score_header = column![
            text("Current ").size(self.font_size),
            text("Progress ").size(self.font_size),
            text("Total ").size(self.font_size),
        ]
        .spacing(self.spacing / 2.0)
        .align_items(Alignment::Start);

        let score_value_text = [
            format!("{} / 1", self.last_score),
            format!("{} / {}", self.total_score.0, self.current.unwrap_or(0) + 1),
            format!("{}", self.total_score.1),
        ];
        let mut score_value = column![]
            .spacing(self.spacing / 2.0)
            .align_items(Alignment::End);
        for i in score_value_text {
            score_value = score_value.push(text(i).size(self.font_size))
        }

        // Final
        let grid = column![
            row![header],
            main,
            row![
                Space::new(Length::FillPortion(10), Length::Fill),
                score_header,
                score_value,
                Space::new(10, Length::Fill),
                next_button,
            ]
            .spacing(self.spacing),
            row![error_log].spacing(self.spacing),
        ]
        .spacing(self.spacing);
        let mut contents = Element::from(grid);
        if self.debug_layout {
            contents = contents.explain(iced::Color::WHITE);
        }
        container(contents)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn theme(&self) -> Theme {
        if self.dark_theme {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

#[cfg(not(target_family = "wasm"))]
async fn pick_file() -> Result<(PathBuf, Arc<([Lang; 2], Vec<Entry>)>), Error> {
    let opt_handle = rfd::AsyncFileDialog::new()
        .set_title("Choose a json file...")
        .add_filter("Json (*.json)", &["json"])
        .add_filter("All files (*.*)", &["*"])
        .pick_file()
        .await;
    match opt_handle {
        Some(handle) => {
            let path = handle.path();
            match async_std::fs::read_to_string(path).await {
                Ok(raw) => match parse(&raw) {
                    Ok(data) => Ok((path.into(), Arc::new(data))),
                    Err(_) => Err(Error::ParseError),
                },
                Err(_) => Err(Error::IoError),
            }
        }
        None => Err(Error::DialogClosed),
    }
}
