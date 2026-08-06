use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;

use log::{error, info};

use image::codecs::jpeg::JpegEncoder;
use image::DynamicImage;
use image::ImageReader;
use image::{ExtendedColorType, ImageDecoder, ImageEncoder};

use fast_image_resize::images::Image;
use fast_image_resize::{PixelType, Resizer};

/// Longest edge, in pixels, that an output image may have.
const MAX_SIZE: u32 = 2048;

/// File extensions treated as JPEG, compared case-insensitively.
const JPEG_EXTENSIONS: [&str; 2] = ["jpg", "jpeg"];

/// List the JPEGs directly inside `dir`, ignoring any subdirectories.
///
/// Reads the directory rather than globbing it: a folder whose own name
/// contains pattern characters (`Photos [2024]`) would otherwise match nothing.
pub fn find_jpegs(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut jpegs: Vec<PathBuf> = Vec::new();

    for entry in fs::read_dir(dir)? {
        let path = entry?.path();

        let is_jpeg = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| {
                JPEG_EXTENSIONS
                    .iter()
                    .any(|jpeg| ext.eq_ignore_ascii_case(jpeg))
            });

        if is_jpeg && path.is_file() {
            jpegs.push(path);
        }
    }

    // read_dir yields entries in arbitrary order; glob returned them sorted, so
    // keep processing order stable and logs comparable between runs.
    jpegs.sort();

    Ok(jpegs)
}

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
    let path_str = path.to_str().ok_or_else(|| {
        error!(path:? = path ; "Invalid path");
        "Path conversion failed"
    })?;

    // Read the dimensions from the header only; a source that needs no
    // shrinking is copied verbatim, so there is no point decoding it.
    let (src_width, src_height) = ImageReader::open(path_str)?.into_dimensions()?;
    info!(src_width, src_height; "Source image size");

    let dst_path = insert_sub_folder(path.clone())?;

    let max_size = std::cmp::max(src_width, src_height);
    if max_size <= MAX_SIZE {
        // Copying preserves the original quality; re-encoding would throw away
        // a generation of detail for no reduction in size.
        info!(path:? = dst_path; "Image already small enough, copying unchanged");
        fs::copy(&path, &dst_path)?;
        return Ok(dst_path.to_string_lossy().to_string());
    }

    let mut decoder = ImageReader::open(path_str)?.into_decoder()?;
    let orientation = decoder.orientation()?;
    let mut src_image = DynamicImage::from_decoder(decoder)?;
    src_image.apply_orientation(orientation);

    // Normalise to RGB8: we always encode JPEG, which has no alpha channel, so
    // this keeps greyscale/RGBA/16-bit sources on a single code path. into_rgb8
    // consumes the decoded image, so an already-RGB8 JPEG - the common case -
    // is moved rather than copied, and the original is freed either way.
    let src_image = DynamicImage::ImageRgb8(src_image.into_rgb8());

    let modifier: f32 = MAX_SIZE as f32 / max_size as f32;

    let dst_width = (src_image.width() as f32 * modifier).floor() as u32;
    let dst_height = (src_image.height() as f32 * modifier).floor() as u32;

    info!(dst_width, dst_height; "Destination image size");

    // src_image is always ImageRgb8, so the destination buffer and the
    // ExtendedColorType::Rgb8 we encode with below are guaranteed to agree.
    let mut dst_image = Image::new(dst_width, dst_height, PixelType::U8x3);

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

    let mut file = File::create(&dst_path)?;
    file.write_all(&result_buf.into_inner()?)?;

    info!(path:? = dst_path; "Image saved");

    Ok(dst_path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use testdir::testdir;

    /// Sorted file names, so assertions do not depend on directory order.
    fn file_names(mut paths: Vec<PathBuf>) -> Vec<String> {
        paths.sort();
        paths
            .iter()
            .map(|path| path.file_name().unwrap().to_string_lossy().to_string())
            .collect()
    }

    #[test]
    fn test_find_jpegs_is_case_insensitive() {
        let dir = testdir!();
        for name in ["lower.jpg", "UPPER.JPG", "mixed.JpEg", "long.jpeg"] {
            fs::write(dir.join(name), b"").expect("Failed to write test file");
        }

        let found = find_jpegs(&dir).expect("find_jpegs failed");

        assert_eq!(
            file_names(found),
            vec!["UPPER.JPG", "long.jpeg", "lower.jpg", "mixed.JpEg"]
        );
    }

    #[test]
    fn test_find_jpegs_ignores_lookalikes() {
        let dir = testdir!();
        fs::write(dir.join("photo.jpg"), b"").expect("Failed to write test file");
        fs::write(dir.join("notes.txt"), b"").expect("Failed to write test file");
        // Matched by the *.jp*g pattern, but not actually a JPEG extension.
        fs::write(dir.join("weird.jpxg"), b"").expect("Failed to write test file");
        // A directory that happens to be named like an image.
        fs::create_dir(dir.join("album.jpg")).expect("Failed to create test dir");

        let found = find_jpegs(&dir).expect("find_jpegs failed");

        assert_eq!(file_names(found), vec!["photo.jpg"]);
    }

    #[test]
    fn test_find_jpegs_in_folder_named_with_glob_metacharacters() {
        let dir = testdir!().join("Photos [2024]");
        fs::create_dir_all(&dir).expect("Failed to create test dir");
        fs::write(dir.join("photo.jpg"), b"").expect("Failed to write test file");

        let found = find_jpegs(&dir).expect("find_jpegs failed");

        assert_eq!(file_names(found), vec!["photo.jpg"]);
    }

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

        // The fixture is 3000x3200, so the long edge is scaled down to MAX_SIZE
        // and the aspect ratio preserved.
        let resized = image::ImageReader::open(&resized_image_path)
            .expect("Failed to open resized image")
            .decode()
            .expect("Failed to decode resized image");
        assert_eq!((resized.width(), resized.height()), (1920, 2048));
    }

    /// Splice a minimal EXIF APP1 segment carrying nothing but an Orientation
    /// tag into a JPEG, straight after the SOI marker. Built by hand so the
    /// fixture is reproducible and readable without an external tool.
    fn with_exif_orientation(jpeg: &[u8], orientation: u8) -> Vec<u8> {
        #[rustfmt::skip]
        let mut app1: Vec<u8> = vec![
            0xFF, 0xE1,             // APP1 marker
            0x00, 0x22,             // segment length, including these 2 bytes
            b'E', b'x', b'i', b'f', 0x00, 0x00,
            0x49, 0x49, 0x2A, 0x00, // TIFF header, little-endian
            0x08, 0x00, 0x00, 0x00, // byte offset of IFD0
            0x01, 0x00,             // IFD0 has one entry
            0x12, 0x01,             // tag 0x0112 = Orientation
            0x03, 0x00,             // type 3 = SHORT
            0x01, 0x00, 0x00, 0x00, // one value
            0x00, 0x00, 0x00, 0x00, // the value itself, patched below
            0x00, 0x00, 0x00, 0x00, // no IFD1
        ];
        app1[28] = orientation;

        let mut out = Vec::with_capacity(jpeg.len() + app1.len());
        out.extend_from_slice(&jpeg[..2]); // SOI
        out.append(&mut app1);
        out.extend_from_slice(&jpeg[2..]);
        out
    }

    #[test]
    fn test_resize_image_applies_exif_orientation() {
        let base_path = testdir!();
        let test_image_path = base_path.join("rotated.jpg");
        let output_path = base_path.join("smol/rotated.jpg");

        // Landscape pixel data that a viewer must rotate 90 degrees clockwise
        // to show upright, i.e. a portrait photo straight off a phone.
        let mut jpeg = Vec::new();
        image::DynamicImage::ImageRgb8(image::RgbImage::new(3000, 2000))
            .write_to(
                &mut std::io::Cursor::new(&mut jpeg),
                image::ImageFormat::Jpeg,
            )
            .expect("Failed to encode test image");
        fs::write(&test_image_path, with_exif_orientation(&jpeg, 6))
            .expect("Failed to write test image");

        let result = resize_image(test_image_path);

        assert!(result.is_ok(), "rotated jpeg failed: {:?}", result.err());
        let output = image::ImageReader::open(&output_path)
            .expect("Failed to open output image")
            .decode()
            .expect("Failed to decode output image");
        assert!(
            output.height() > output.width(),
            "EXIF orientation was not applied: got {}x{}, expected portrait",
            output.width(),
            output.height()
        );
    }

    #[test]
    fn test_resize_image_copies_already_small_image_unchanged() {
        let base_path = testdir!();
        let test_image_path = base_path.join("small.jpg");
        let output_path = base_path.join("smol/small.jpg");

        image::DynamicImage::ImageRgb8(image::RgbImage::new(200, 100))
            .save(&test_image_path)
            .expect("Failed to write small test image");
        let original_bytes = fs::read(&test_image_path).expect("Failed to read test image");

        let result = resize_image(test_image_path);

        assert!(result.is_ok(), "small jpeg failed: {:?}", result.err());
        let output_bytes = fs::read(&output_path).expect("Failed to read output image");
        assert!(
            output_bytes == original_bytes,
            "already-small image should be copied byte-for-byte \
             (original {} bytes, output {} bytes)",
            original_bytes.len(),
            output_bytes.len()
        );
    }

    #[test]
    fn test_resize_image_greyscale_jpeg() {
        let base_path = testdir!();
        let test_image_path = base_path.join("greyscale.jpg");

        image::DynamicImage::ImageLuma8(image::GrayImage::new(3000, 2000))
            .save(&test_image_path)
            .expect("Failed to write greyscale test image");

        let result = resize_image(test_image_path);

        assert!(result.is_ok(), "greyscale jpeg failed: {:?}", result.err());
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
