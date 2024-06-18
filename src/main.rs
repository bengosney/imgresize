use glob::glob;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::{cell::RefCell, rc::Rc};

use imgsize::ThreadPool;
use std::path::PathBuf;

use image::codecs::jpeg::JpegEncoder;
use image::io::Reader as ImageReader;
use image::{ExtendedColorType, ImageEncoder};

use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, Resizer};

use fltk::{app, dialog, prelude::*};

mod ui {
    fl2rust_macro::include_ui!("src/window.fl");
}

fn resize_image<P: Into<PathBuf>>(img_path: P) {
    // Read source image from file
    let mut path: PathBuf = img_path.into();
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

#[derive(Debug, Clone)]
struct Model {
    count: usize,
    max: usize,
}
impl Model {
    fn init() -> Self {
        Self { count: 0, max: 0 }
    }
    pub fn inc(&mut self) -> usize {
        self.count += 1;
        self.max = std::cmp::max(self.max, self.count);
        self.max - self.count
    }
    pub fn dec(&mut self) -> usize {
        self.count -= 1;
        self.max - self.count
    }
}

fn main() {
    app::GlobalState::<Model>::new(Model::init());
    let app = app::App::default();
    let pool = ThreadPool::new(2);

    let mut ui = ui::UserInterface::make_window();
    let selected_dir = Rc::new(RefCell::new(None));

    ui.resize_btn.set_callback({
        let selected_dir = selected_dir.clone();
        let mut ui = ui.clone();
        move |_| {
            if let Some(selected_dir) = &*selected_dir.borrow() {
                let glob_path = format!("{}/*.jpg", selected_dir);
                let state = app::GlobalState::<Model>::get();
                for file in glob(&glob_path).expect("Failed to read glob pattern") {
                    let path = file.unwrap();
                    let count = state.with(|model| model.inc());
                    ui.progress
                        .set_maximum(state.with(|model| model.max) as f64);
                    ui.progress.set_value(count as f64);

                    println!("{}", path.to_str().unwrap());

                    pool.execute({
                        let mut ui = ui.clone();
                        move || {
                            resize_image(path);
                            let count = app::GlobalState::<Model>::get().with(|model| model.dec());
                            ui.progress.set_value(count as f64);
                        }
                    });
                }
            }
        }
    });

    ui.select_dir_btn.set_callback({
        let selected_dir = selected_dir.clone();
        move |_| {
            let mut dialog =
                dialog::NativeFileChooser::new(dialog::NativeFileChooserType::BrowseDir);
            dialog.show();
            *selected_dir.borrow_mut() = Some(dialog.filename().to_str().unwrap().to_string());
            ui.selected_dir
                .set_label(selected_dir.borrow().as_ref().unwrap());
        }
    });

    app.run().unwrap();
}
