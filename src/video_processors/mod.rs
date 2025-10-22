use crate::video_processors::{
    blended::BlendedSettings,
    colored::ColoredSettings,
    priority::PrioritySettings,
    stable::StableSettings,
};

pub mod blended;
pub mod colored;
pub mod priority;
pub mod stable;
pub mod utils;

/// Defines the visual effects.
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq)]
pub enum EffectMode {
    /// A stable trail effect that burns the motion into the frame.
    Stable,
    /// A blended trail effect that creates a ghostly, semi-transparent trail.
    Blended,
    /// A colored trail effect that leaves a trail of a specific color or a rainbow.
    Colored,
    /// A priority-based effect that keeps the brightest or darkest pixels.
    Priority,
}

/// Holds all the settings.
pub struct EffectSettings {
    pub mode: EffectMode,
    /// If `true`, the audio from the input video will be copied to the output.
    pub preserve_audio: bool,
    /// The threshold for detecting motion between frames (0.0 to 1.0).
    /// A lower value means more sensitivity to motion.
    pub motion_threshold_percent: f32,
    /// If true, a correction pass is applied to reduce glowing edges on moving objects.
    pub use_edge_correction: bool,
    /// The number of frames to skip between trail updates. 1 applies the effect on every frame.
    pub n_frames_step: usize,

    pub stable: StableSettings,
    pub blended: BlendedSettings,
    pub colored: ColoredSettings,
    pub priority: PrioritySettings,
}

/// A helper struct to hold the calculated per-frame decay amounts for each effect.
pub struct CalculatedDecay {
    pub stable: f32,
    pub blended: f32,
    pub colored: f32,
    pub priority: f32,
}
