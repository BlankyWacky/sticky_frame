use image::{RgbImage, RgbaImage};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;
use std::process::Command;
use video_rs::{
    decode::DecoderBuilder,
    encode::{Encoder, Settings},
    Frame,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Effect Settings
    let burn_in_factor: f32 = 0.4;
    let diff_r_threshold_percent: f32 = 0.1;
    let diff_g_threshold_percent: f32 = 0.1;
    let diff_b_threshold_percent: f32 = 0.1;
    let preserve_audio: bool = true;

    // Init
    video_rs::init()?;

    let source_path = Path::new("input.mp4");
    let temp_video_path = Path::new("temp_output.mp4");
    let final_output_path = Path::new("output.mp4");

    // Video processing scope
    {
        println!("Opening decoder for: {}", source_path.display());
        let mut decoder = DecoderBuilder::new(source_path).build()?;

        let (width, height) = decoder.size();
        let frame_rate = decoder.frame_rate();
        let total_frames = decoder.frames()?;

        println!("Video properties: {}x{} @ {} fps", width, height, frame_rate);
        println!("Creating encoder for temporary video: {}", temp_video_path.display());

        let mut encoder = Encoder::new(
            temp_video_path,
            Settings::preset_h264_yuv420p(width as usize, height as usize, false),
        )?;

        let mut canvas: Option<RgbaImage> = None;
        let mut previous_frame: Option<RgbaImage> = None;

        // Progress bar setup
        let pb = ProgressBar::new(total_frames as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} frames ({eta})")?
            .progress_chars("#+-"));

        // Iterate over frames from the video.
        for (frame_index, frame_result) in decoder.decode_iter().enumerate() {
            if let Ok((timestamp, frame)) = frame_result {
                let rgb_frame: RgbImage = RgbImage::from_raw(width, height, frame.into_raw_vec_and_offset().0).unwrap();
                let current_frame_image: RgbaImage = image::DynamicImage::ImageRgb8(rgb_frame).to_rgba8();

                if frame_index == 0 {
                    canvas = Some(current_frame_image.clone());
                    previous_frame = Some(current_frame_image.clone());
                } else {
                    let mut new_canvas_data = canvas.as_ref().unwrap().clone();
                    let prev_frame_data = previous_frame.as_ref().unwrap();

                    let r_thresh = (255.0 * diff_r_threshold_percent) as i16;
                    let g_thresh = (255.0 * diff_g_threshold_percent) as i16;
                    let b_thresh = (255.0 * diff_b_threshold_percent) as i16;

                    for (x, y, current_pixel) in current_frame_image.enumerate_pixels() {
                        let prev_pixel = prev_frame_data.get_pixel(x, y);

                        let diff_r = (current_pixel[0] as i16 - prev_pixel[0] as i16).abs();
                        let diff_g = (current_pixel[1] as i16 - prev_pixel[1] as i16).abs();
                        let diff_b = (current_pixel[2] as i16 - prev_pixel[2] as i16).abs();

                        if diff_r > r_thresh || diff_g > g_thresh || diff_b > b_thresh {
                            let old_pixel = new_canvas_data.get_pixel_mut(x, y);
                            let new_r = (old_pixel[0] as f32 * (1.0 - burn_in_factor) + current_pixel[0] as f32 * burn_in_factor) as u8;
                            let new_g = (old_pixel[1] as f32 * (1.0 - burn_in_factor) + current_pixel[1] as f32 * burn_in_factor) as u8;
                            let new_b = (old_pixel[2] as f32 * (1.0 - burn_in_factor) + current_pixel[2] as f32 * burn_in_factor) as u8;
                            *old_pixel = image::Rgba([new_r, new_g, new_b, 255]);
                        }
                    }
                    canvas = Some(new_canvas_data);
                    previous_frame = Some(current_frame_image);
                }

                if let Some(canvas_to_encode) = &canvas {
                    let rgb_canvas = image::DynamicImage::ImageRgba8(canvas_to_encode.clone()).to_rgb8();
                    let frame_to_encode: Frame = Frame::from_shape_vec((height as usize, width as usize, 3), rgb_canvas.clone().into_raw())
                            .expect("Could not create ndarray from image buffer");
                    encoder.encode(&frame_to_encode, timestamp)?;
                }
                
                pb.inc(1);
            } else {
                break;
            }
        }

        pb.finish_with_message("Video processing complete.");
        encoder.finish()?;
    } // Encoder, Decoder go out of scope here

    // Audio merging with FFmpeg if needed
    if preserve_audio {
        let status = Command::new("ffmpeg")
            .arg("-y") // Overwrite output file if it exists
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
            .status()?;

        if status.success() {
            println!("Successfully merged audio and video.");
        } else {
            eprintln!("FFmpeg command failed. The temporary video file is available at: {}", temp_video_path.display());
            return Err("FFmpeg command failed".into());
        }

        // Remove temporary video file
        fs::remove_file(temp_video_path)?;
    }
    else {
        // If not preserving audio, rename temp video to final output
        fs::rename(temp_video_path, final_output_path)?;
    }

    println!("Done! Final video saved to {}", final_output_path.display());

    Ok(())
}
