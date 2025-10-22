# Sticky Frame - Video Trail Effects

A command-line application written in Rust to create various motion trail effects on videos. It works by comparing consecutive frames and applying a persistent visual effect to pixels that change significantly.

### Demo (Stable Effect)

| Before | After |
| :----: | :---: |
| ![Before](assets/demo_input.gif) | ![After](assets/demo_output.gif) |

*Settings used for the demo above are from a previous version of the project.*

## Features

-   **Multiple Trail Effects**: Choose from four different visual effects:
    -   **Stable**: A classic "burn-in" effect for moving objects.
    -   **Blended**: A ghostly, semi-transparent trail.
    -   **Colored**: A trail with a static color or a cycling rainbow.
    -   **Priority**: An effect that keeps the brightest or darkest pixels.
-   **Highly Customizable**: Each effect has its own set of parameters that can be tweaked to achieve the desired look.
-   **Audio Preservation**: The audio from the original video is automatically merged into the final processed video.
-   **Console Progress Bar**: Shows processing progress, including ETA, in the console.

## Prerequisites

Before running this project, you must have the following installed:

1.  **Rust and Cargo**: [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
2.  **FFmpeg**: This is required for merging the audio back into the processed video. You can download it from [https://ffmpeg.org/download.html](https://ffmpeg.org/download.html). Make sure it's accessible from your system's PATH.

## How to Use

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/BlankyWacky/sticky_frame
    cd sticky_frame
    ```

2.  **Add your video:**
    Place the video you want to process into the root of the project directory and name it `input.mp4`.

3.  **Configure the effect (optional):**
    Open `src/main.rs` and modify the `EffectSettings` struct to your liking.

4.  **Run the program:**
    Execute the following command in your terminal:
    ```bash
    cargo run --release
    ```
    This will build and run the application in release mode for optimal performance.

5.  **Get the result:**
    The final video, including audio, will be saved as `output.mp4` in the project root.

## Configuration

You can customize the video effect by changing the fields in the `EffectSettings` struct at the top of the `main` function in `src/main.rs`.

### General Settings

-   `mode: EffectMode`: The main visual effect to apply. Options are `EffectMode::Stable`, `EffectMode::Blended`, `EffectMode::Colored`, and `EffectMode::Priority`.
-   `preserve_audio: bool`: If `true`, the audio from the input video will be copied to the output.
-   `motion_threshold_percent: f32`: The threshold for detecting motion between frames (0.0 to 1.0). A lower value means more sensitivity to motion.
-   `use_edge_correction: bool`: If `true`, a correction pass is applied to reduce glowing edges on moving objects.
-   `n_frames_step: usize`: The number of frames to skip between trail updates. `1` applies the effect on every frame.

### `Stable` Effect Settings

-   `burn_in_factor: f32`: The opacity of new trails when they are stamped onto the canvas (0.0 to 1.0).
-   `tracer_duration_ms: Option<u32>`: The duration a trail should last, in milliseconds. If `None`, the trail is permanent.

### `Blended` Effect Settings

-   `blend_factor: f32`: The blend factor for combining the canvas and the current frame (0.0 to 1.0).
-   `tracer_duration_ms: Option<u32>`: The duration it takes for a static background to fade in to full clarity, in milliseconds. If `None`, the background will not fade in.

### `Colored` Effect Settings

-   `color: image::Rgba<u8>`: The static color of the trails if `rainbow_mode` is `false`.
-   `rainbow_mode: bool`: If `true`, the trail color will cycle through the rainbow.
-   `rainbow_speed: f32`: The speed at which the rainbow color cycles. Higher is faster.
-   `tracer_opacity: f32`: The opacity of the stamped trail (0.0 to 1.0).
-   `tracer_duration_ms: Option<u32>`: The duration a trail should last, in milliseconds. If `None`, the trail is permanent.

### `Priority` Effect Settings

-   `mode: PriorityMode`: The comparison logic to use. Options are `PriorityMode::Lightest` and `PriorityMode::Darkest`.
-   `tracer_duration_ms: Option<u32>`: The duration melded pixels should last before fading back to the live video, in milliseconds. If `None`, the effect is permanent.

## License

This project is licensed under the MIT License.