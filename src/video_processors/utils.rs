use image::RgbaImage;

use crate::video_processors::EffectSettings;

/// Linearly interpolates a `current` u8 value towards a `target` u8 value by a fixed `amount`.
pub fn move_towards(current: u8, target: u8, amount: f32) -> u8 {
    if amount <= 0.0 {
        return current;
    }
    let current_f = current as f32;
    let target_f = target as f32;
    if current_f < target_f {
        (current_f + amount).min(target_f) as u8
    } else if current_f > target_f {
        (current_f - amount).max(target_f) as u8
    } else {
        current
    }
}

/// Creates a motion mask by comparing the `current` and `prev` frames.
pub fn create_motion_mask(current: &RgbaImage, prev: &RgbaImage, threshold: i16) -> Vec<bool> {
    let (width, height) = current.dimensions();
    let mut mask = vec![false; (width * height) as usize];
    for (x, y, current_pixel) in current.enumerate_pixels() {
        let prev_pixel = prev.get_pixel(x, y);
        let diff_r = (current_pixel[0] as i16 - prev_pixel[0] as i16).abs();
        let diff_g = (current_pixel[1] as i16 - prev_pixel[1] as i16).abs();
        let diff_b = (current_pixel[2] as i16 - prev_pixel[2] as i16).abs();
        if diff_r > threshold || diff_g > threshold || diff_b > threshold {
            mask[(y * width + x) as usize] = true;
        }
    }
    mask
}

/// Applies compositing and edge correction to the output frame.
pub fn apply_compositing_and_correction(
    mut output_frame: RgbaImage,
    canvas: &RgbaImage,
    current_frame: &RgbaImage,
    motion_mask: &[bool],
    settings: &EffectSettings,
) -> RgbaImage {
    let (width, height) = output_frame.dimensions();
    for (x, y, pixel) in output_frame.enumerate_pixels_mut() {
        if !motion_mask[(y * width + x) as usize] {
            *pixel = *canvas.get_pixel(x, y);
        }
    }

    if settings.use_edge_correction {
        let mut corrected_frame = output_frame.clone();
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let idx = (y * width + x) as usize;
                if !motion_mask[idx] {
                    let is_touching_motion =
                        motion_mask[idx - 1]
                            || motion_mask[idx + 1]
                            || motion_mask[idx - width as usize]
                            || motion_mask[idx + width as usize];
                    if is_touching_motion {
                        *corrected_frame.get_pixel_mut(x, y) = *current_frame.get_pixel(x, y);
                    }
                }
            }
        }
        return corrected_frame;
    }

    output_frame
}

/// Converts a color from HSV to RGB.
///
/// # Arguments
///
/// * `h` - Hue (0.0 to 360.0).
/// * `s` - Saturation (0.0 to 1.0).
/// * `v` - Value (0.0 to 1.0).
///
/// # Returns
///
/// A tuple of `(u8, u8, u8)` representing the RGB color.
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r_prime, g_prime, b_prime) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r_prime + m) * 255.0) as u8,
        ((g_prime + m) * 255.0) as u8,
        ((b_prime + m) * 255.0) as u8,
    )
}
