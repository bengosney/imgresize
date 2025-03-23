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

fn insert_sub_folder(path: PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let file_name: String = match path.file_name() {
        Some(file_name) => file_name.to_string_lossy().to_string(),
        None => {
            error!(path:? = path; "Invalid path");
            return Err("Invalid path".into());
        }
    };
    let mut path = path;
    path.pop();
    path.push("smol");
    fs::create_dir_all(&path)?;
    path.push(file_name);
    Ok(path)
}

pub fn resize_image(path: PathBuf) -> Result<String, Box<dyn std::error::Error>> {
    info!(
        path:? = path,
        thread_id:? = thread::current().id();
        "Resizing image"
    );
    // Read source image from file
    let src_image = ImageReader::open(path.to_str().ok_or_else(|| {
        error!(path:? = path ; "Invalid path");
        "Path conversion failed"
    })?)?
    .decode()?;

    let path = insert_sub_folder(path)?;

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

    let mut file = File::create(path.clone())?;
    file.write_all(&result_buf.into_inner()?)?;

    info!(path:? = path; "Image saved");

    return Ok(path.to_string_lossy().to_string());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use testdir::testdir;

    #[test]
    fn test_insert_sub_folder_valid_path() {
        let base_path = testdir!();
        let test_path = base_path.join("test_image.jpg");
        let expected_path = base_path.join("smol/test_image.jpg");

        let result = insert_sub_folder(test_path.clone());

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_path);
    }

    #[test]
    fn test_insert_sub_folder_invalid_path() {
        let invalid_path = PathBuf::from("");

        let result = insert_sub_folder(invalid_path);

        assert!(result.is_err());
    }

    #[test]
    fn test_insert_sub_folder_creates_directory() {
        let base_path = testdir!();
        let test_path = base_path.join("test_image.jpg");
        let expected_dir = base_path.join("smol");

        let result = insert_sub_folder(test_path.clone());

        assert!(result.is_ok());
        assert!(expected_dir.exists());
    }

    #[test]
    fn test_resize_image_success() {
        let base_path = testdir!();
        let test_image_path = base_path.join("test_image.jpg");
        let resized_image_path = base_path.join("smol/test_image.jpg");

        match fs::copy(
            PathBuf::from("tests/images/test_image.jpg"),
            test_image_path.clone(),
        ) {
            Ok(_) => {}
            Err(e) => panic!("Failed to copy test image: {}", e),
        }

        let result = resize_image(test_image_path);

        assert!(result.is_ok());
        assert!(resized_image_path.exists());
    }

    #[test]
    fn test_resize_image_invalid_path() {
        let invalid_path = PathBuf::from("invalid/path/to/image.jpg");

        let result = resize_image(invalid_path);

        assert!(result.is_err());
    }

    #[test]
    fn test_resize_image_non_image_file() {
        let non_image_path = PathBuf::from("tests/images/not_an_image.txt");

        let result = resize_image(non_image_path);

        assert!(result.is_err());
    }
}
