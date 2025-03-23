use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;
use std::thread;

use log::{error, info};

use image::codecs::jpeg::JpegEncoder;
use image::ImageReader;
use image::{ExtendedColorType, ImageEncoder};

use fast_image_resize::images::Image;
use fast_image_resize::{IntoImageView, Resizer};

pub fn resize_image(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    info!(
        path:? = path,
        thread_id:? = thread::current().id();
        "Resizing image"
    );
    // Read source image from file
    let mut path = path;
    let src_image = ImageReader::open(path.to_str().ok_or_else(|| {
        error!(path:? = path ; "Invalid path");
        "Path conversion failed"
    })?)?
    .decode()?;

    let filename: String = match path.file_name() {
        Some(name) => name.to_string_lossy().to_string(),
        None => {
            error!(path:? = path; "Invalid path");
            return Err("Invalid path".into());
        }
    };
    path.pop();
    path.push("smol");

    fs::create_dir_all(match path.to_str() {
        Some(path) => path,
        None => {
            error!(path:? = path; "Invalid path");
            return Err("Invalid path".into());
        }
    })?;

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

    let pixel_type = match src_image.pixel_type() {
        Some(pixel_type) => pixel_type,
        None => {
            error!("Pixel type not found");
            return Err("Pixel type not found".into());
        }
    };

    let mut dst_image = Image::new(dst_width, dst_height, pixel_type);

    // Create Resizer instance and resize source image
    // into buffer of destination image
    let mut resizer = Resizer::new();
    resizer.resize(&src_image, &mut dst_image, None)?;

    // Write destination image to file
    let mut result_buf = BufWriter::new(Vec::new());
    JpegEncoder::new(&mut result_buf).write_image(
        dst_image.buffer(),
        dst_width,
        dst_height,
        ExtendedColorType::Rgb8,
    )?;

    let mut file = File::create(path)?;
    file.write_all(&result_buf.into_inner()?)?;
    info!(filename:? = filename; "Image saved");

    return Ok(filename);
}
