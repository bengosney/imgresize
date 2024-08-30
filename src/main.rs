use glob::glob;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::sync::atomic::AtomicI32;
use std::sync::Arc;

use imgsize::ThreadPool;
use std::path::PathBuf;

use image::codecs::jpeg::JpegEncoder;
use image::io::Reader as ImageReader;
use image::{ExtendedColorType, ImageEncoder};

use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, Resizer};

use iced::widget::{button, column, progress_bar, text};
use iced::{Alignment, Application, Command, Element, Length, Settings, Subscription};

use native_dialog::FileDialog;

#[derive(Default)]
struct ImageResizer {
    completed: i32,
    total: i32,
    path: Option<PathBuf>,
    pool: ThreadPool,
    completed_tracker: Arc<AtomicI32>,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    OpenFileDialog,
    ResizeImages,
    Tick,
}

fn resize_image(path: PathBuf) {
    // Read source image from file
    let mut path = path;
    let src_image = ImageReader::open(path.to_str().unwrap())
        .unwrap()
        .decode()
        .unwrap();

    let filename: String = path.file_name().unwrap().to_string_lossy().into_owned();
    path.pop();
    path.push("smol");

    fs::create_dir_all(path.to_str().unwrap()).unwrap();

    path.push(filename.clone());

    let src_width = src_image.width();
    let src_height = src_image.height();

    let max_size = std::cmp::max(src_width, src_height);
    let modifier: f32 = 2048.0 / max_size as f32;

    println!("Source image: {}x{}", src_width, src_height);

    // Create container for data of destination image
    let dst_width = (src_width as f32 * modifier).floor() as u32;
    let dst_height = (src_height as f32 * modifier).floor() as u32;

    println!("Destination image: {}x{}", dst_width, dst_height);

    let mut dst_image = Image::new(dst_width, dst_height, src_image.pixel_type().unwrap());

    // Create Resizer instance and resize source image
    // into buffer of destination image
    let mut resizer = Resizer::new();
    resizer.resize(&src_image, &mut dst_image, None).unwrap();

    // Write destination image to file
    let mut result_buf = BufWriter::new(Vec::new());
    JpegEncoder::new(&mut result_buf)
        .write_image(
            dst_image.buffer(),
            dst_width,
            dst_height,
            ExtendedColorType::Rgb8,
        )
        .unwrap();

    let mut file = File::create(path).unwrap();
    file.write_all(&result_buf.into_inner().unwrap()).unwrap();
    println!("Image {} saved", filename);
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
                            resize_image(path);
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
