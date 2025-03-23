use glob::glob;
use std::path::PathBuf;

use log::{debug, error, info};

use iced::widget::{button, column, progress_bar, text};
use iced::{Alignment, Application, Command, Element, Length, Settings};

use native_dialog::FileDialog;

use imgsize::resize_image;

#[derive(PartialEq)]
enum ProcesingState {
    Idle,
    Processing,
    Completed,
}

impl Default for ProcesingState {
    fn default() -> Self {
        ProcesingState::Idle
    }
}

#[derive(Default)]
struct ImageResizer {
    completed: i32,
    total: i32,
    path: Option<PathBuf>,
    processing_state: ProcesingState,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    OpenFileDialog,
    ResizeImages,
    ProgressIncrement,
    ProcesingComplete,
}

#[derive(Debug, Clone, Default)]
struct UiFlags {}

impl Application for ImageResizer {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Theme = iced::Theme;
    type Flags = UiFlags;

    fn new(_flags: UiFlags) -> (Self, Command<Message>) {
        (ImageResizer::default(), Command::none())
    }

    fn title(&self) -> String {
        String::from("Image Resizer")
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::OpenFileDialog => {
                debug!("Opening file dialog");
                self.path = match FileDialog::new().show_open_single_dir() {
                    Ok(Some(path)) => {
                        info!(path:? = path; "Selected folder");
                        Some(path)
                    }
                    Ok(None) => {
                        info!("No folder selected");
                        None
                    }
                    Err(e) => {
                        error!(error:? = e; "Error opening file dialog");
                        None
                    }
                };

                self.processing_state = ProcesingState::Idle;
                self.total = 0;
                self.completed = 0;
                Command::none()
            }
            Message::ResizeImages => {
                self.total = 0;
                self.completed = 0;
                let glob_path = format!("{}/*.jp*g", self.path.clone().unwrap().to_str().unwrap());

                let files: Vec<_> = glob(&glob_path)
                    .expect("Failed to read glob pattern")
                    .filter_map(Result::ok)
                    .collect();
                self.total = files.len() as i32;
                info!(total_images = self.total; "Total files to resize");
                self.processing_state = ProcesingState::Processing;

                let commands: Vec<_> = files
                    .into_iter()
                    .map(|file| {
                        Command::perform(resize_image_async(file), |_| Message::ProgressIncrement)
                    })
                    .collect();

                Command::batch(commands)
            }
            Message::ProgressIncrement => {
                self.completed += 1;
                debug!(completed = self.completed; "Incrementing progress");

                if self.completed == self.total {
                    Command::perform(async {}, |_| Message::ProcesingComplete)
                } else {
                    Command::none()
                }
            }
            Message::ProcesingComplete => {
                info!("Resizing completed");
                self.processing_state = ProcesingState::Completed;
                self.path = None;
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<Self::Message> {
        let message = match self.processing_state {
            ProcesingState::Idle => match self.path.clone() {
                Some(path) => format!("Selected: {:?}", truncate(path.to_str().unwrap(), 22)),
                None => "Select a folder with images to resize".to_string(),
            },
            ProcesingState::Processing => format!("Progress: {} of {}", self.completed, self.total),
            ProcesingState::Completed => "Resizing completed".to_string(),
        };

        let select_folder_button = if self.processing_state != ProcesingState::Processing {
            button("Select Folder").on_press(Message::OpenFileDialog)
        } else {
            button("Select Folder")
        };

        let resize_button =
            if self.path.is_some() && self.processing_state != ProcesingState::Processing {
                button("Resize Images").on_press(Message::ResizeImages)
            } else {
                button("Resize Images")
            };

        column![
            select_folder_button.width(Length::Fill),
            resize_button.width(Length::Fill),
            text(message).width(Length::Shrink),
            progress_bar(0.0..=self.total as f32, self.completed as f32).width(Length::Fill),
        ]
        .spacing(10)
        .padding(20)
        .width(Length::Fill)
        .align_items(Alignment::Center)
        .into()
    }
}

fn truncate(s: &str, len: usize) -> String {
    if s.len() > len {
        format!("...{}", &s[(s.len() - len)..])
    } else {
        s.to_string()
    }
}

async fn resize_image_async(path: PathBuf) {
    resize_image(path);
}

fn main() -> iced::Result {
    ImageResizer::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(400.0, 175.0),
            resizable: false,
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
}
