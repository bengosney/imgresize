## 0.6.0 (2025-03-24)

### Feat

- **tests**: add unit tests for ImageResizer functionality and improve state debugging
- **resize**: refactor image resizing logic to include sub-folder creation
- **logging**: integrate structured logging and command-line options for log level
- **logging**: add logging functionality using the log crate

### Refactor

- **tests**: update tests to use testdir for dynamic path handling
- **errors**: improve error handling and logging

## 0.5.0 (2025-03-22)

### Feat

- **ui**: enhance image resizer with processing state management and UI updates
- **async**: swap to async and tokio to handle threading
- **threadding**: use rayon rather than my custom threadding

### Refactor

- **image**: implement image resizing functionality and refactor related code
- **main**: improve code readability by formatting and organizing imports

## 0.4.0 (2025-01-15)

### Feat

- **CARGO**: update dependacies and mimimum rust version

## 0.3.1 (2024-10-08)

### Fix

- **files**: glob both jpg and jpeg file extentions

## 0.3.0 (2024-08-30)

### Feat

- **gui**: reimplement the gui in iced
- **ThreadPool**: implement default and the set the thread count to the number of cores
- **gui**: update the progress bar as we resize
- **GUI**: pass around the path in a slighly better way, I'm quite happy with it
- **resize**: use threads for image resizing
- **GUI**: poorly organised, single threaded but working gui
- **POC**: Proof of concept using fast_image_resize
- **POC**: inital proof of concept for a image resizing

### Perf

- **build**: don't bother with stack trace on crash

## 0.2.0 (2024-08-26)

### Feat

- **gui**: update the progress bar as we resize
- **GUI**: pass around the path in a slightly better way, I'm quite happy with it
- **resize**: use threads for image resizing
- **GUI**: poorly organized, single threaded but working gui

## 0.1.0 (2024-06-15)

### Feat

- **POC**: Proof of concept using fast_image_resize
- **POC**: inital proof of concept for a image resizing
