use image::{
    RgbImage,
    RgbaImage,
};
use indicatif::{
    ProgressBar,
    ProgressStyle,
};
use log::info;
use std::{
    fs,
    path::Path,
    process::Command,
};
use video_rs::{
    Frame,
    decode::DecoderBuilder,
    encode::{
        Encoder,
        Settings,
    },
};

use crate::video_processors::{
    CalculatedDecay,
    EffectMode,
    EffectSettings,
    blended::{
        BlendedSettings,
        process_blended_frame,
    },
    colored::{
        ColoredSettings,
        process_colored_frame,
    },
    priority::{
        PriorityMode,
        PrioritySettings,
        process_priority_frame,
    },
    stable::{
        StableSettings,
        process_stable_frame,
    },
};

mod video_processors;

fn main() {
    // Effect Settings
    let settings = EffectSettings {
        // General
        mode: EffectMode::Colored,
        preserve_audio: true,
        motion_threshold_percent: 0.1,
        use_edge_correction: false,
        n_frames_step: 1,

        // Mode-Specific
        stable: StableSettings {
            burn_in_factor: 0.8,
            tracer_duration_ms: Some(3000),
        },
        blended: BlendedSettings {
            blend_factor: 0.5,
            tracer_duration_ms: Some(10000),
        },
        colored: ColoredSettings {
            color: image::Rgba([255, 255, 255, 255]),
            rainbow_mode: true,
            rainbow_speed: 5.0,
            tracer_opacity: 1.0,
            tracer_duration_ms: Some(4000),
        },
        priority: PrioritySettings {
            mode: PriorityMode::Lightest,
            tracer_duration_ms: Some(2000),
        },
    };

    // Init video-rs
    if let Err(e) = video_rs::init() {
        eprintln!("Failed to initialize video_rs: {}", e);
        return;
    }

    let source_path = Path::new("input.mp4");
    let temp_video_path = Path::new("temp_output.mp4");
    let final_output_path = Path::new("output.mp4");

    // Video Processing
    // This block handles the decoding, encoding and processing, needs to be in a separate scope for audio handling later.
    if let Err(e) = (|| -> Result<(), Box<dyn std::error::Error>> {
        // Decoding
        println!("Opening decoder for: {}", source_path.display());
        let mut decoder = DecoderBuilder::new(source_path).build()?;
        let (width, height) = decoder.size();
        let frame_rate = decoder.frame_rate();
        let total_frames = decoder.frames()?;
        println!("Video properties: {}x{} @ {} fps", width, height, frame_rate);

        // Calculate the per-frame decay amount for each effect mode based on the configured duration.
        let decay = CalculatedDecay {
            stable: duration_ms_to_decay(settings.stable.tracer_duration_ms, frame_rate),
            blended: duration_ms_to_decay(settings.blended.tracer_duration_ms, frame_rate),
            colored: duration_ms_to_decay(settings.colored.tracer_duration_ms, frame_rate),
            priority: duration_ms_to_decay(settings.priority.tracer_duration_ms, frame_rate),
        };

        // Create encoder
        let mut encoder = Encoder::new(
            temp_video_path,
            Settings::preset_h264_yuv420p(width as usize, height as usize, false),
        )?;

        // Frame Processing Loop
        let mut canvas: Option<RgbaImage> = None;
        let mut previous_frame: Option<RgbaImage> = None;
        let mut rainbow_hue: f32 = 0.0;

        // Progress Bar Setup
        let pb = ProgressBar::new(total_frames as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} frames ({eta})")?
                .progress_chars("#+-"),
        );

        for (frame_index, frame_result) in decoder.decode_iter().enumerate() {
            if let Ok((timestamp, frame)) = frame_result {
                let rgb_frame: RgbImage =
                    RgbImage::from_raw(width, height, frame.into_raw_vec_and_offset().0)
                        .ok_or("Failed to create RGB image from frame")?;
                let current_frame_image: RgbaImage =
                    image::DynamicImage::ImageRgb8(rgb_frame).to_rgba8();

                // Initialize the canvas and previous_frame on the first frame
                if frame_index == 0 {
                    canvas = Some(if settings.mode == EffectMode::Colored {
                        RgbaImage::new(width, height)
                    } else {
                        current_frame_image.clone()
                    });
                    previous_frame = Some(current_frame_image.clone());
                }

                let canvas_data = canvas.as_mut().unwrap();
                let prev_frame_data = previous_frame.as_ref().unwrap();
                let should_update_canvas =
                    frame_index > 0 && frame_index % settings.n_frames_step == 0;

                // Effect Processing
                let output_frame = match settings.mode {
                    EffectMode::Stable => process_stable_frame(
                        canvas_data,
                        &current_frame_image,
                        prev_frame_data,
                        &settings,
                        &decay,
                        should_update_canvas,
                    ),
                    EffectMode::Blended => process_blended_frame(
                        canvas_data,
                        &current_frame_image,
                        prev_frame_data,
                        &settings,
                        &decay,
                        should_update_canvas,
                    ),
                    EffectMode::Colored => process_colored_frame(
                        canvas_data,
                        &current_frame_image,
                        prev_frame_data,
                        &settings,
                        &decay,
                        should_update_canvas,
                        &mut rainbow_hue,
                    ),
                    EffectMode::Priority => process_priority_frame(
                        canvas_data,
                        &current_frame_image,
                        &settings,
                        &decay,
                        should_update_canvas,
                    ),
                };

                previous_frame = Some(current_frame_image);

                // Encode the processed frame
                let rgb_output = image::DynamicImage::ImageRgba8(output_frame).to_rgb8();
                let frame_to_encode: Frame =
                    Frame::from_shape_vec((height as usize, width as usize, 3), rgb_output.into_raw())
                        .expect("Could not create ndarray from image buffer");
                encoder.encode(&frame_to_encode, timestamp)?;

                pb.inc(1);
            } else {
                break;
            }
        }
        pb.finish_with_message("Video processing complete.");
        Ok(())
    })() { // Execute the closure
        eprintln!("An error occurred during video processing: {}", e);
        return;
    }

    // Audio Processing
    // If preserve_audio is enabled, use FFmpeg to copy the audio from the source video
    // to the processed video.
    if settings.preserve_audio {
        let status = Command::new("ffmpeg")
            .arg("-y")
            .arg("-i")
            .arg(temp_video_path)
            .arg("-i")
            .arg(source_path)
            .arg("-c:v")
            .arg("copy")
            .arg("-c:a")
            .arg("copy")
            .arg("-map")
            .arg("0:v:0")
            .arg("-map")
            .arg("1:a:0")
            .arg(final_output_path)
            .status(); // Well, this is a bit ugly but video_rs doesn't handle audio yet.

        match status {
            Ok(s) if s.success() => {
                // Clean up the temporary video file
                if let Err(e) = fs::remove_file(temp_video_path) {
                    eprintln!("Failed to remove temporary file: {}", e);
                }
            }
            _ => {
                eprintln!("FFmpeg command failed. The video was saved without audio.");
                if let Err(e) = fs::rename(temp_video_path, final_output_path) {
                    eprintln!("Failed to rename temporary file: {}", e);
                }
            }
        }
    } else {
        // No audio preservation, just rename the temp file to final output
        if let Err(e) = fs::rename(temp_video_path, final_output_path) {
            eprintln!("Failed to rename temporary file: {}", e);
        }
    }

    info!("Done! Final video saved to {}", final_output_path.display());
}

/// Converts a duration in milliseconds to a per-frame decay amount.
fn duration_ms_to_decay(duration_ms: Option<u32>, frame_rate: f32) -> f32 {
    match duration_ms {
        None => 0.0, // No decay
        Some(0) => 255.0, // Instant decay
        Some(ms) => {
            if frame_rate < 1.0 {
                return 255.0; // Avoid division by zero or very small numbers
            }
            let total_frames = (ms as f32 / 1000.0) * frame_rate;
            255.0 / total_frames
        }
    }
}
