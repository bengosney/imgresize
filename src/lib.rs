use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;
use std::thread;

use log::{debug, error, info, warn};

use image::codecs::jpeg::JpegEncoder;
use image::ImageReader;
use image::{ExtendedColorType, ImageEncoder};

use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, Resizer};

pub fn resize_image(path: PathBuf) {
    info!(
        path:? = path,
        thread_id:? = thread::current().id();
        "Resizing image"
    );
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

    info!(src_width, src_height; "Source image size");

    // Create container for data of destination image
    let dst_width = (src_width as f32 * modifier).floor() as u32;
    let dst_height = (src_height as f32 * modifier).floor() as u32;

    info!(dst_width, dst_height; "Destination image size");

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
    info!(filename:? = filename; "Image saved");
}
