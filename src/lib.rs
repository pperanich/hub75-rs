#![no_std]
#![doc = include_str!("../README.md")]

//! # HUB75 Embassy Driver
//!
//! A high-performance, embassy-compatible driver for HUB75 RGB LED matrix displays
//! with embedded-graphics support.
//!
//! ## Features
//!
//! - Full HUB75 protocol implementation
//! - Embassy-rs async/await integration
//! - embedded-graphics DrawTarget support
//! - Binary Code Modulation (BCM) for high color depth
//! - Configurable panel sizes and color depths
//! - Animation support
//! - Double buffering for smooth updates
//! - DMA support (where available)
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use hub75_embassy::{Hub75Display, Hub75Pins};
//! use embassy_executor::Spawner;
//! use embedded_graphics::prelude::*;
//!
//! #[embassy_executor::main]
//! async fn main(spawner: Spawner) {
//!     let pins = Hub75Pins::new(/* your pins */);
//!     let mut display = Hub75Display::<_, 64, 32, 6>::new(pins);
//!
//!     // Start background refresh task
//!     spawner.spawn(display_task(display)).unwrap();
//! }
//! ```

pub mod animation;
pub mod color;
pub mod display;
pub mod frame_buffer;
pub mod pins;

/// Macro to simplify pin error handling
macro_rules! pin_op {
    ($op:expr) => {
        $op.map_err(|_| crate::Hub75Error::PinError)?
    };
}

pub(crate) use pin_op;

// Error types (moved from error.rs for consolidation)

/// Errors that can occur when using the HUB75 driver
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Hub75Error {
    /// Pin operation failed
    PinError,
    /// Invalid coordinates provided
    InvalidCoordinates,
    /// Invalid color value
    InvalidColor,
    /// Animation error
    AnimationError(AnimationError),
    /// Buffer overflow
    BufferOverflow,
}

/// Animation-specific errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AnimationError {
    /// Animation is too fast for the refresh rate
    TooFast,
    /// Invalid animation data
    InvalidData,
    /// Animation duration is invalid
    InvalidDuration,
}

impl From<AnimationError> for Hub75Error {
    fn from(err: AnimationError) -> Self {
        Hub75Error::AnimationError(err)
    }
}

// Re-export main types
pub use animation::{Animation, AnimationEffect, AnimationState};
pub use color::Hub75Color;
pub use display::Hub75Display;
pub use frame_buffer::Hub75FrameBuffer;
pub use pins::{Hub75AddressPins, Hub75ControlPins, Hub75Pins, Hub75RgbPins};

// Re-export commonly used types from dependencies
pub use embassy_time::{Duration, Instant, Timer};
pub use embedded_hal::digital::OutputPin;

#[cfg(feature = "embedded-graphics")]
pub use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::{Rgb565, RgbColor},
    Pixel,
};

// Common panel size type aliases
pub type Hub75<P, const W: usize, const H: usize, const C: usize> = Hub75Display<P, W, H, C>;

// Specific panel sizes with common color depths
pub type Hub75_32x16<P, const COLOR_BITS: usize> = Hub75<P, 32, 16, COLOR_BITS>;
pub type Hub75_64x32<P, const COLOR_BITS: usize> = Hub75<P, 64, 32, COLOR_BITS>;
pub type Hub75_64x64<P, const COLOR_BITS: usize> = Hub75<P, 64, 64, COLOR_BITS>;
pub type Hub75_128x64<P, const COLOR_BITS: usize> = Hub75<P, 128, 64, COLOR_BITS>;
