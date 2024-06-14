use image::imageops::FilterType;
//use image::ImageFormat;
//use std::fmt;
//use std::fs::File;
//use std::time::{Duration, Instant};

fn main() {
    println!("Opening image...");
    let img = image::open("./images/src/PXL_20240608_064409362.jpg").unwrap();
    println!("Image opened successfully");

    const SIZE: u32 = 2092;
    let small = img.resize(SIZE, SIZE, FilterType::Lanczos3);
    println!("Image resized successfully");
    let result = small.save("./images/dest/one.jpg");
    match result {
        Ok(_) => println!("Image saved successfully"),
        Err(e) => println!("Error saving image: {}", e),
    }
}
