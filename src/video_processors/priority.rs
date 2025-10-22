use image::RgbaImage;

use crate::video_processors::{
    CalculatedDecay,
    EffectSettings,
    utils::move_towards,
};

/// Defines the comparison logic for the `Priority` effect, need to allow dead code to remain P R E T T Y
#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum PriorityMode {
    /// Keeps the brighter of the two pixels
    Lightest,
    /// Keeps the darker of the two pixels
    Darkest,
}

/// Configuration for the `Priority` effect.
pub struct PrioritySettings {
    pub mode: PriorityMode,
    /// The duration a trail should last, in milliseconds, none for permanent.
    pub tracer_duration_ms: Option<u32>,
}

/// Processes a single frame for the `Priority` effect.
/// This effect creates trails by comparing the brightness of the canvas pixel and the current
/// frame pixel, keeping either the lightest or the darkest of the two.
pub fn process_priority_frame(
    canvas: &mut RgbaImage,
    current_frame: &RgbaImage,
    settings: &EffectSettings,
    decay: &CalculatedDecay,
    should_update_canvas: bool,
) -> RgbaImage {
    if should_update_canvas {
        let decay_amount = decay.priority;
        for (x, y, canvas_pixel) in canvas.enumerate_pixels_mut() {
            let current_pixel = *current_frame.get_pixel(x, y);

            // Decay the canvas pixel towards the current frame pixel
            if decay_amount > 0.0 {
                *canvas_pixel = image::Rgba([
                    move_towards(canvas_pixel[0], current_pixel[0], decay_amount),
                    move_towards(canvas_pixel[1], current_pixel[1], decay_amount),
                    move_towards(canvas_pixel[2], current_pixel[2], decay_amount),
                    255,
                ]);
            }

            // Apply the priority logic
            let canvas_brightness = canvas_pixel[0] as u16 + canvas_pixel[1] as u16 + canvas_pixel[2] as u16;
            let current_brightness = current_pixel[0] as u16 + current_pixel[1] as u16 + current_pixel[2] as u16;

            match settings.priority.mode {
                PriorityMode::Lightest => {
                    if current_brightness > canvas_brightness {
                        *canvas_pixel = current_pixel;
                    }
                }
                PriorityMode::Darkest => {
                    if current_brightness < canvas_brightness {
                        *canvas_pixel = current_pixel;
                    }
                }
            }
        }
    }

    canvas.clone()
}
