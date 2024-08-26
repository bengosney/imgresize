# IMG Resizer

This is a simple and fast image resizing tool.
- Simple - It only resizes to a reasonable size for social media
- Fast   - Thanks to fast_image_resize and it's use of SIMD, it's _fast_

## Technical

It's written in rust, uses fast_image_resize for resizing and fltk as GUI library.
I've tested it using rustc/cargo 1.80.0 and run on Linux Mint and Windows 11.

### Build

It's all standard rust/cargo stuff so the normal `cargo build` should work.
