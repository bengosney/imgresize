use glob::glob;
use std::io;
use std::path::PathBuf;

use log::{debug, error, info, LevelFilter};
use structopt::StructOpt;
use structured_logger::{json::new_writer, Builder};

use iced::widget::{button, column, progress_bar, text};
use iced::{Alignment, Application, Command, Element, Length, Settings};

use native_dialog::FileDialog;

use imgsize::resize_image;

#[derive(PartialEq, Debug)]
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
                let glob_path = match &self.path {
                    Some(path) => path.join("*.jp*g").to_string_lossy().to_string(),
                    None => {
                        error!("Trying to resize with no path selected");
                        return Command::none();
                    }
                };

                let files: Vec<_> = match glob(&glob_path) {
                    Ok(files) => files.filter_map(Result::ok).collect(),
                    Err(e) => {
                        error!(error:? = e; "Failed to read glob pattern");
                        return Command::none();
                    }
                };
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
            ProcesingState::Idle => self
                .path
                .as_ref()
                .and_then(|path| {
                    path.to_str()
                        .map(|path_str| format!("Selected folder: {}", truncate(path_str, 50)))
                })
                .unwrap_or_else(|| "Select a folder with images to resize".to_string()),
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

async fn resize_image_async(path: PathBuf) -> String {
    match resize_image(path) {
        Ok(image_path) => image_path,
        Err(e) => {
            error!(error:? = e; "Error resizing image");
            format!("Error resizing image: {}", e)
        }
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "image-resizer", about = "Resize images in a folder")]
struct Opt {
    #[structopt(short, long, default_value = "warn")]
    log_level: LevelFilter,
}

fn main() -> iced::Result {
    let opt = Opt::from_args();
    Builder::with_level(&opt.log_level.as_str())
        .with_target_writer("imgsize", new_writer(io::stdout()))
        .init();

    ImageResizer::run(Settings {
        window: iced::window::Settings {
            size: iced::Size::new(400.0, 175.0),
            resizable: false,
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let resizer = ImageResizer::default();
        assert_eq!(resizer.completed, 0);
        assert_eq!(resizer.total, 0);
        assert_eq!(resizer.path, None);
        assert_eq!(resizer.processing_state, ProcesingState::Idle);
    }

    // #[test]
    // fn test_open_file_dialog() {
    //     let mut resizer = ImageResizer::default();
    //     let _ = resizer.update(Message::OpenFileDialog);

    //     assert_eq!(resizer.processing_state, ProcesingState::Idle);
    //     assert_eq!(resizer.completed, 0);
    //     assert_eq!(resizer.total, 0);
    // }

    #[test]
    fn test_resize_images_no_path() {
        let mut resizer = ImageResizer::default();
        let _ = resizer.update(Message::ResizeImages);

        assert_eq!(resizer.processing_state, ProcesingState::Idle);
        assert_eq!(resizer.completed, 0);
        assert_eq!(resizer.total, 0);
    }

    #[test]
    fn test_progress_increment() {
        let mut resizer = ImageResizer {
            completed: 0,
            total: 5,
            path: Some(PathBuf::from("/some/path")),
            processing_state: ProcesingState::Processing,
        };

        let _ = resizer.update(Message::ProgressIncrement);

        assert_eq!(resizer.completed, 1);
        assert_eq!(resizer.processing_state, ProcesingState::Processing);
    }

    #[test]
    fn test_processing_complete() {
        let mut resizer = ImageResizer {
            completed: 5,
            total: 5,
            path: Some(PathBuf::from("/some/path")),
            processing_state: ProcesingState::Processing,
        };

        let _ = resizer.update(Message::ProcesingComplete);

        assert_eq!(resizer.processing_state, ProcesingState::Completed);
        assert_eq!(resizer.path, None);
    }

    #[test]
    fn test_truncate_short_string() {
        let input = "short";
        let result = truncate(input, 10);
        assert_eq!(result, "short");
    }

    #[test]
    fn test_truncate_long_string() {
        let input = "this_is_a_very_long_string";
        let result = truncate(input, 10);
        assert_eq!(result, "...ong_string");
    }
}
