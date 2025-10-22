use image::{
    Pixel,
    RgbaImage,
};

use crate::video_processors::{
    CalculatedDecay,
    EffectSettings,
    utils::{
        create_motion_mask,
        hsv_to_rgb,
        move_towards,
    },
};

/// Configuration for the `Colored` effect.
pub struct ColoredSettings {
    /// The static color of the trails if `rainbow_mode` is `false`.
    pub color: image::Rgba<u8>,
    /// If `true`, the trail color will cycle through the rainbow.
    pub rainbow_mode: bool,
    /// The speed at which the rainbow color cycles. Higher is faster. `1.0` is a good starting point.
    pub rainbow_speed: f32,
    /// The opacity of the stamped trail (0.0 to 1.0).
    /// - `0.0` makes the trail completely transparent.
    /// - `1.0` makes the trail completely solid.
    pub tracer_opacity: f32,
    /// The duration a trail should last, in milliseconds, none for permanent.
    pub tracer_duration_ms: Option<u32>,
}

/// Processes a single frame for the `Colored` effect.
/// This effect creates a colored trail where motion is detected. The trail can be a static color
/// or a cycling rainbow. The opacity and duration of the trail can be configured.
pub fn process_colored_frame(
    canvas: &mut RgbaImage,
    current_frame: &RgbaImage,
    prev_frame: &RgbaImage,
    settings: &EffectSettings,
    decay: &CalculatedDecay,
    should_trail: bool,
    rainbow_hue: &mut f32,
) -> RgbaImage {
    let motion_thresh = (255.0 * settings.motion_threshold_percent) as i16;
    let (width, _height) = canvas.dimensions();
    let motion_mask = create_motion_mask(current_frame, prev_frame, motion_thresh);
    let decay_amount = decay.colored;

    // Decay the canvas pixels over time
    if decay_amount > 0.0 {
        for pixel in canvas.pixels_mut() {
            *pixel = image::Rgba([
                move_towards(pixel[0], 0, decay_amount),
                move_towards(pixel[1], 0, decay_amount),
                move_towards(pixel[2], 0, decay_amount),
                255,
            ]);
        }
    }

    // Add new trails in areas of motion
    if should_trail {
        let trail_color = if settings.colored.rainbow_mode {
            let (r, g, b) = hsv_to_rgb(*rainbow_hue, 1.0, 1.0);
            *rainbow_hue = (*rainbow_hue + settings.colored.rainbow_speed) % 360.0;
            image::Rgba([r, g, b, 255])
        } else {
            settings.colored.color
        };

        for (x, y, canvas_pixel) in canvas.enumerate_pixels_mut() {
            if motion_mask[(y * width + x) as usize] {
                let opacity = settings.colored.tracer_opacity;
                canvas_pixel.blend(&image::Rgba([
                    (canvas_pixel[0] as f32 * (1.0 - opacity) + trail_color[0] as f32 * opacity) as u8,
                    (canvas_pixel[1] as f32 * (1.0 - opacity) + trail_color[1] as f32 * opacity) as u8,
                    (canvas_pixel[2] as f32 * (1.0 - opacity) + trail_color[2] as f32 * opacity) as u8,
                    255,
                ]));
            }
        }
    }

    // Combine the canvas and the current frame to create the final output
    let mut output_frame = current_frame.clone();
    for (x, y, output_pixel) in output_frame.enumerate_pixels_mut() {
        let canvas_pixel = canvas.get_pixel(x, y);
        let r = (output_pixel[0] as u16 + canvas_pixel[0] as u16).min(255) as u8;
        let g = (output_pixel[1] as u16 + canvas_pixel[1] as u16).min(255) as u8;
        let b = (output_pixel[2] as u16 + canvas_pixel[2] as u16).min(255) as u8;
        *output_pixel = image::Rgba([r, g, b, 255]);
    }

    output_frame
}
