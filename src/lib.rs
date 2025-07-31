#![no_std]
#![doc = include_str!("../README.md")]

//! # HUB75 Driver
//!
//! A high-performance, generic async driver for HUB75 RGB LED matrix displays
//! with embedded-graphics support.
//!
//! ## Features
//!
//! - Full HUB75 protocol implementation
//! - Generic async/await support (works with Embassy, RTIC, etc.)
//! - embedded-graphics DrawTarget support
//! - Binary Code Modulation (BCM) for high color depth
//! - Configurable panel sizes and color depths
//! - Animation support with frame-based timing
//! - Double buffering for smooth updates
//! - DelayNs trait for flexible timing providers
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use hub75::{Hub75Display, Hub75Pins, Hub75RgbPins, Hub75AddressPins, Hub75ControlPins};
//! use embedded_hal_async::delay::DelayNs;
//! use embedded_graphics::prelude::*;
//! use embedded_graphics::primitives::{Rectangle, PrimitiveStyleBuilder};
//! use embedded_graphics::pixelcolor::Rgb565;
//!
//! # async fn example(pin: impl embedded_hal::digital::OutputPin + Clone, mut delay: impl DelayNs) -> Result<(), hub75::Hub75Error> {
//! // Configure pins for your microcontroller
//! let pins = Hub75Pins {
//!     rgb: Hub75RgbPins {
//!         r1: pin.clone(), g1: pin.clone(), b1: pin.clone(),
//!         r2: pin.clone(), g2: pin.clone(), b2: pin.clone(),
//!     },
//!     address: Hub75AddressPins {
//!         a: pin.clone(), b: pin.clone(), c: pin.clone(),
//!         d: Some(pin.clone()), e: None,
//!     },
//!     control: Hub75ControlPins {
//!         clk: pin.clone(), lat: pin.clone(), oe: pin,
//!     },
//! };
//!
//! // Create a 64x32 display with 6-bit color depth
//! let mut display = Hub75Display::<_, 64, 32, 6>::new(pins)?;
//!
//! // Enable double buffering for smooth updates
//! display.set_double_buffering(true);
//!
//! // Draw a red rectangle using embedded-graphics
//! Rectangle::new(Point::new(10, 10), Size::new(20, 12))
//!     .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::RED).build())
//!     .draw(&mut display)
//!     .unwrap();
//!
//! // Swap buffers and render to the display
//! display.swap_buffers();
//! display.render_frame(&mut delay).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Platform Examples
//!
//! ### Embassy (nRF52/RP2040)
//!
//! ```rust,no_run
//! # use embedded_hal_async::delay::DelayNs;
//! # struct EmbassyDelay;
//! # impl DelayNs for EmbassyDelay { async fn delay_ns(&mut self, _ns: u32) {} }
//! use hub75::Hub75Display;
//!
//! # async fn embassy_example() -> Result<(), hub75::Hub75Error> {
//! # let pins = todo!(); // Your pin configuration
//! let mut display = Hub75Display::<_, 64, 32, 6>::new(pins)?;
//! let mut delay = EmbassyDelay; // embassy_time::Delay in real code
//!
//! // Use Embassy's delay implementation
//! display.render_frame(&mut delay).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### RTIC or other async runtimes
//!
//! ```rust,no_run
//! # use embedded_hal_async::delay::DelayNs;
//! # struct CustomDelay;
//! # impl DelayNs for CustomDelay {
//! #     async fn delay_ns(&mut self, _ns: u32) { /* implementation */ }
//! # }
//! use hub75::Hub75Display;
//!
//! # async fn rtic_example() -> Result<(), hub75::Hub75Error> {
//! # let pins = todo!(); // Your pin configuration
//! let mut display = Hub75Display::<_, 64, 32, 6>::new(pins)?;
//! let mut delay = CustomDelay; // Your delay implementation
//!
//! display.render_frame(&mut delay).await?;
//! # Ok(())
//! # }
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
pub use embedded_hal::digital::OutputPin;
pub use embedded_hal_async::delay::DelayNs;

#[cfg(feature = "embedded-graphics")]
pub use embedded_graphics_core::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::{Rgb565, RgbColor},
    Pixel,
};

/// Generic HUB75 display type alias for convenience
///
/// This is a shorthand for `Hub75Display<P, W, H, C>` where:
/// - `P`: Pin type implementing `OutputPin`
/// - `W`: Display width in pixels
/// - `H`: Display height in pixels  
/// - `C`: Color depth in bits (typically 4, 6, or 8)
///
/// # Example
/// ```rust,no_run
/// use hub75::{Hub75, Hub75Pins};
/// use embedded_hal::digital::OutputPin;
///
/// // Create a 64x32 display with 6-bit color depth
/// type MyDisplay = Hub75<impl OutputPin, 64, 32, 6>;
/// ```
pub type Hub75<P, const W: usize, const H: usize, const C: usize> = Hub75Display<P, W, H, C>;

/// 32x16 HUB75 display type alias
///
/// Common small panel size, often used for simple displays or as building blocks
/// for larger displays. Supports configurable color depth.
///
/// # Example
/// ```rust,no_run
/// use hub75::{Hub75_32x16, Hub75Pins};
/// use embedded_hal::digital::OutputPin;
///
/// // 32x16 display with 4-bit color depth (16 colors per channel)
/// type SmallDisplay = Hub75_32x16<impl OutputPin, 4>;
/// ```
pub type Hub75_32x16<P, const COLOR_BITS: usize> = Hub75<P, 32, 16, COLOR_BITS>;

/// 64x32 HUB75 display type alias
///
/// Very common panel size, widely available and well-supported. Good balance
/// between resolution and refresh rate. Often used in commercial displays.
///
/// # Example
/// ```rust,no_run
/// use hub75::{Hub75_64x32, Hub75Pins};
/// use embedded_hal::digital::OutputPin;
///
/// // 64x32 display with 6-bit color depth (64 colors per channel)
/// type StandardDisplay = Hub75_64x32<impl OutputPin, 6>;
/// ```
pub type Hub75_64x32<P, const COLOR_BITS: usize> = Hub75<P, 64, 32, COLOR_BITS>;

/// 64x64 HUB75 display type alias
///
/// Square format panel, popular for decorative displays and art installations.
/// Higher pixel count requires more processing power and memory.
///
/// # Example
/// ```rust,no_run
/// use hub75::{Hub75_64x64, Hub75Pins};
/// use embedded_hal::digital::OutputPin;
///
/// // 64x64 display with 8-bit color depth (256 colors per channel)
/// type SquareDisplay = Hub75_64x64<impl OutputPin, 8>;
/// ```
pub type Hub75_64x64<P, const COLOR_BITS: usize> = Hub75<P, 64, 64, COLOR_BITS>;

/// 128x64 HUB75 display type alias
///
/// Large panel size with high resolution. Requires significant processing power
/// and memory. Often used for text displays and detailed graphics.
///
/// # Example
/// ```rust,no_run
/// use hub75::{Hub75_128x64, Hub75Pins};
/// use embedded_hal::digital::OutputPin;
///
/// // 128x64 display with 6-bit color depth for good performance
/// type LargeDisplay = Hub75_128x64<impl OutputPin, 6>;
/// ```
pub type Hub75_128x64<P, const COLOR_BITS: usize> = Hub75<P, 128, 64, COLOR_BITS>;
