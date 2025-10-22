use image::RgbaImage;

use crate::video_processors::{
    CalculatedDecay,
    EffectSettings,
    utils::{
        create_motion_mask,
        move_towards,
    },
};

/// Configuration for the `Blended` effect.
pub struct BlendedSettings {
    /// The blend factor for combining the canvas and the current frame (0.0 to 1.0).
    /// - `0.0` makes the live action completely transparent (ghostly).
    /// - `1.0` makes the live action completely solid.
    /// - `0.5` creates a semi-transparent effect.
    pub blend_factor: f32,
    /// The duration it takes for a static background to fade in to full clarity, in milliseconds.
    /// If `None`, the background will not fade in.
    pub tracer_duration_ms: Option<u32>,
}

/// Processes a single frame for the `Blended` effect.
/// This effect creates a ghostly, semi-transparent trail by blending the current frame with a
/// persistent canvas. The canvas gradually clarifies in static areas, creating a fade-in effect
/// for the background.
pub fn process_blended_frame(
    canvas: &mut RgbaImage,
    current_frame: &RgbaImage,
    prev_frame: &RgbaImage,
    settings: &EffectSettings,
    decay: &CalculatedDecay,
    should_update_canvas: bool,
) -> RgbaImage {
    let motion_thresh = (255.0 * settings.motion_threshold_percent) as i16;
    let (width, height) = canvas.dimensions();
    let motion_mask = create_motion_mask(current_frame, prev_frame, motion_thresh);
    let mut output_frame = RgbaImage::new(width, height);
    let clarify_amount = decay.blended;

    for (x, y, canvas_pixel) in canvas.enumerate_pixels_mut() {
        if should_update_canvas && !motion_mask[(y * width + x) as usize] && clarify_amount > 0.0 {
            let current_pixel = *current_frame.get_pixel(x, y);
            *canvas_pixel = image::Rgba([
                move_towards(canvas_pixel[0], current_pixel[0], clarify_amount),
                move_towards(canvas_pixel[1], current_pixel[1], clarify_amount),
                move_towards(canvas_pixel[2], current_pixel[2], clarify_amount),
                255,
            ]);
        }

        let current_pixel = *current_frame.get_pixel(x, y);
        let clarity = settings.blended.blend_factor;

        // Blend the canvas and the current frame to create the final output
        *output_frame.get_pixel_mut(x, y) = image::Rgba([
            (canvas_pixel[0] as f32 * (1.0 - clarity) + current_pixel[0] as f32 * clarity) as u8,
            (canvas_pixel[1] as f32 * (1.0 - clarity) + current_pixel[1] as f32 * clarity) as u8,
            (canvas_pixel[2] as f32 * (1.0 - clarity) + current_pixel[2] as f32 * clarity) as u8,
            255,
        ]);
    }

    output_frame
}
