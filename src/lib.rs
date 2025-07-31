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
pub mod error;
pub mod frame_buffer;
pub mod pins;

// Re-export main types
pub use animation::{Animation, AnimationEffect, AnimationState};
pub use color::Hub75Color;
pub use display::Hub75Display;
pub use error::Hub75Error;
pub use frame_buffer::Hub75FrameBuffer;
pub use pins::{Hub75ControlPins, Hub75Pins, Hub75RgbPins};

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
pub type Hub75_32x16<P, const COLOR_BITS: usize> = Hub75Display<P, 32, 16, COLOR_BITS>;
pub type Hub75_64x32<P, const COLOR_BITS: usize> = Hub75Display<P, 64, 32, COLOR_BITS>;
pub type Hub75_64x64<P, const COLOR_BITS: usize> = Hub75Display<P, 64, 64, COLOR_BITS>;
pub type Hub75_128x64<P, const COLOR_BITS: usize> = Hub75Display<P, 128, 64, COLOR_BITS>;
