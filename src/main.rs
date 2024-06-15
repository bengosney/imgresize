use std::io::BufWriter;
use std::fs::File;
use std::io::Write;

use image::codecs::jpeg::JpegEncoder;
use image::io::Reader as ImageReader;
use image::{ExtendedColorType, ImageEncoder};

use fast_image_resize::{IntoImageView, Resizer};
use fast_image_resize::images::Image;

fn main() {
    // Read source image from file
    let src_image = ImageReader::open("./images/src/PXL_20240608_064409362.jpg")
        .unwrap()
        .decode()
        .unwrap();

    let src_width = src_image.width();
    let src_height = src_image.height();

    let max_size = std::cmp::max(src_width, src_height);
    let modifier: f32 = 2048.0 / max_size as f32;

    println!("Source image: {}x{}", src_width, src_height);

    // Create container for data of destination image
    let dst_width = (src_width as f32 * modifier).floor() as u32;
    let dst_height = (src_height as f32 * modifier).floor() as u32;

    println!("Destination image: {}x{}", dst_width, dst_height);

    let mut dst_image = Image::new(
        dst_width,
        dst_height,
        src_image.pixel_type().unwrap(),
    );

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

    let mut file = File::create("./images/dest/one.jpg").unwrap();
    file.write_all(&result_buf.into_inner().unwrap()).unwrap();
}