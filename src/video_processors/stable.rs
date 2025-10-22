use image::RgbaImage;

use crate::video_processors::{
    CalculatedDecay,
    EffectSettings,
    utils::{
        apply_compositing_and_correction,
        create_motion_mask,
        move_towards,
    },
};

/// Configuration for the `Stable` effect.
pub struct StableSettings {
    /// The opacity of new trails when they are stamped onto the canvas (0.0 to 1.0).
    /// - `0.0` makes the trail completely transparent.
    /// - `1.0` makes the trail completely solid.
    pub burn_in_factor: f32,
    /// The duration a trail should last, in milliseconds, none for permanent.
    pub tracer_duration_ms: Option<u32>,
}

/// Processes a single frame for the `Stable` effect.
/// This effect creates a stable trail by "burning in" motion into a persistent canvas.
/// The canvas gradually fades back to the current frame in static areas.
pub fn process_stable_frame(
    canvas: &mut RgbaImage,
    current_frame: &RgbaImage,
    prev_frame: &RgbaImage,
    settings: &EffectSettings,
    decay: &CalculatedDecay,
    should_update_canvas: bool,
) -> RgbaImage {
    let motion_thresh = (255.0 * settings.motion_threshold_percent) as i16;
    let (width, _height) = canvas.dimensions();
    let motion_mask = create_motion_mask(current_frame, prev_frame, motion_thresh);

    if should_update_canvas {
        let decay_amount = decay.stable;
        for (x, y, canvas_pixel) in canvas.enumerate_pixels_mut() {
            let is_in_motion = motion_mask[(y * width + x) as usize];
            if is_in_motion {
                let current_pixel = *current_frame.get_pixel(x, y);
                // Blend for burn-in
                let factor = settings.stable.burn_in_factor;
                *canvas_pixel = image::Rgba([
                    (canvas_pixel[0] as f32 * (1.0 - factor) + current_pixel[0] as f32 * factor) as u8,
                    (canvas_pixel[1] as f32 * (1.0 - factor) + current_pixel[1] as f32 * factor) as u8,
                    (canvas_pixel[2] as f32 * (1.0 - factor) + current_pixel[2] as f32 * factor) as u8,
                    255,
                ]);
            } else if decay_amount > 0.0 {
                // Decay the canvas towards the current frame in static areas
                let current_pixel = *current_frame.get_pixel(x, y);
                *canvas_pixel = image::Rgba([
                    move_towards(canvas_pixel[0], current_pixel[0], decay_amount),
                    move_towards(canvas_pixel[1], current_pixel[1], decay_amount),
                    move_towards(canvas_pixel[2], current_pixel[2], decay_amount),
                    255,
                ]);
            }
        }
    }

    apply_compositing_and_correction(
        current_frame.clone(),
        canvas,
        current_frame,
        &motion_mask,
        settings,
    )
}
