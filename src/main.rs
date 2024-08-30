use glob::glob;
use std::sync::atomic::AtomicI32;
use std::sync::Arc;

use std::path::PathBuf;

use iced::widget::{button, column, progress_bar, text};
use iced::{Alignment, Application, Command, Element, Length, Settings, Subscription};

use native_dialog::FileDialog;

mod resize;
mod threads;

#[derive(Default)]
struct ImageResizer {
    completed: i32,
    total: i32,
    path: Option<PathBuf>,
    pool: threads::ThreadPool,
    completed_tracker: Arc<AtomicI32>,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    OpenFileDialog,
    ResizeImages,
    Tick,
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

    fn subscription(&self) -> Subscription<Message> {
        iced::time::every(std::time::Duration::from_millis(500)).map(|_| Message::Tick)
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        match message {
            Message::OpenFileDialog => {
                self.path = FileDialog::new().show_open_single_dir().unwrap();
            }
            Message::ResizeImages => {
                self.completed_tracker
                    .store(0, std::sync::atomic::Ordering::SeqCst);
                self.total = 0;
                self.completed = 0;
                let glob_path = format!("{}/*.jpg", self.path.clone().unwrap().to_str().unwrap());
                for file in glob(&glob_path).expect("Failed to read glob pattern") {
                    self.total += 1;
                    let finished = self.completed_tracker.clone();
                    let path = file.unwrap();
                    self.pool.execute({
                        move || {
                            println!("{:?}", path.display());
                            resize::resize_image(path);
                            finished.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        }
                    });
                }
            }
            Message::Tick => {
                self.completed = self
                    .completed_tracker
                    .load(std::sync::atomic::Ordering::SeqCst);
            }
        };
        Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        column![
            button("Select Folder")
                .on_press(Message::OpenFileDialog)
                .width(Length::Fill),
            button("Resize Images")
                .on_press(Message::ResizeImages)
                .width(Length::Fill),
            text(format!("Progress: {} of {}", self.completed, self.total)),
            progress_bar(0.0..=self.total as f32, self.completed as f32).width(Length::Fill),
        ]
        .spacing(10)
        .padding(20)
        .width(Length::Fill)
        .align_items(Alignment::Center)
        .into()
    }
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
