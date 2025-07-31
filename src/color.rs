//! Color management for HUB75 displays

use core::fmt;

/// Trait for converting between color formats
pub trait ColorConvert<T> {
    /// Convert from another color format
    fn from_color(color: T) -> Self;
    /// Convert to another color format
    fn to_color(self) -> T;
}

/// RGB color representation for HUB75 displays
///
/// This struct represents a color with configurable bit depth for each channel.
/// The bit depth is specified as a const generic parameter and determines the
/// maximum value for each color component.
///
/// # Examples
///
/// ```rust
/// use hub75::Hub75Color;
///
/// // 4-bit color (16 levels per channel)
/// let red: Hub75Color<4> = Hub75Color::new(15, 0, 0);
///
/// // 6-bit color (64 levels per channel)
/// let green: Hub75Color<6> = Hub75Color::new(0, 63, 0);
///
/// // 8-bit color (256 levels per channel)
/// let blue: Hub75Color<8> = Hub75Color::new(0, 0, 255);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Hub75Color<const BITS: usize> {
    /// Red color component (0 to MAX_VALUE)
    ///
    /// The maximum value depends on the bit depth:
    /// - 4-bit: 0-15
    /// - 6-bit: 0-63  
    /// - 8-bit: 0-255
    pub r: u8,

    /// Green color component (0 to MAX_VALUE)
    ///
    /// The maximum value depends on the bit depth:
    /// - 4-bit: 0-15
    /// - 6-bit: 0-63
    /// - 8-bit: 0-255
    pub g: u8,

    /// Blue color component (0 to MAX_VALUE)
    ///
    /// The maximum value depends on the bit depth:
    /// - 4-bit: 0-15
    /// - 6-bit: 0-63
    /// - 8-bit: 0-255
    pub b: u8,
}

impl<const BITS: usize> Hub75Color<BITS> {
    /// Maximum value for this bit depth (computed at compile time)
    pub const MAX_VALUE: u8 = (1 << BITS) - 1;

    /// Create a new color with the specified RGB values
    /// Values are automatically clamped to the bit depth
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: if r > Self::MAX_VALUE {
                Self::MAX_VALUE
            } else {
                r
            },
            g: if g > Self::MAX_VALUE {
                Self::MAX_VALUE
            } else {
                g
            },
            b: if b > Self::MAX_VALUE {
                Self::MAX_VALUE
            } else {
                b
            },
        }
    }

    /// Create a black color (all components zero)
    pub const fn black() -> Self {
        Self { r: 0, g: 0, b: 0 }
    }

    /// Create a white color (all components at maximum)
    pub const fn white() -> Self {
        Self {
            r: Self::MAX_VALUE,
            g: Self::MAX_VALUE,
            b: Self::MAX_VALUE,
        }
    }

    /// Create a red color
    pub const fn red() -> Self {
        Self {
            r: Self::MAX_VALUE,
            g: 0,
            b: 0,
        }
    }

    /// Create a green color
    pub const fn green() -> Self {
        Self {
            r: 0,
            g: Self::MAX_VALUE,
            b: 0,
        }
    }

    /// Create a blue color
    pub const fn blue() -> Self {
        Self {
            r: 0,
            g: 0,
            b: Self::MAX_VALUE,
        }
    }

    /// Get the bit value for a specific bit plane
    pub fn get_bit(&self, bit_plane: usize) -> (bool, bool, bool) {
        if bit_plane >= BITS {
            return (false, false, false);
        }

        let mask = 1 << bit_plane;
        (
            (self.r & mask) != 0,
            (self.g & mask) != 0,
            (self.b & mask) != 0,
        )
    }

    /// Convert from 8-bit RGB values, scaling to the target bit depth
    pub const fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        if BITS >= 8 {
            Self::new(r, g, b)
        } else {
            let shift = 8 - BITS;
            Self::new(r >> shift, g >> shift, b >> shift)
        }
    }

    /// Convert to 8-bit RGB values, scaling from the current bit depth
    pub const fn to_rgb8(&self) -> (u8, u8, u8) {
        if BITS >= 8 {
            (self.r, self.g, self.b)
        } else {
            let shift = 8 - BITS;
            (self.r << shift, self.g << shift, self.b << shift)
        }
    }
}

impl<const BITS: usize> Default for Hub75Color<BITS> {
    fn default() -> Self {
        Self::black()
    }
}

impl<const BITS: usize> fmt::Display for Hub75Color<BITS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RGB({}, {}, {})", self.r, self.g, self.b)
    }
}

#[cfg(feature = "embedded-graphics")]
mod embedded_graphics_support {
    use super::*;
    use embedded_graphics_core::pixelcolor::{Rgb565, Rgb888, RgbColor};

    impl<const BITS: usize> ColorConvert<Rgb565> for Hub75Color<BITS> {
        fn from_color(color: Rgb565) -> Self {
            Self::from_rgb8(
                (color.r() as u16 * 255 / 31) as u8,
                (color.g() as u16 * 255 / 63) as u8,
                (color.b() as u16 * 255 / 31) as u8,
            )
        }

        fn to_color(self) -> Rgb565 {
            let (r, g, b) = self.to_rgb8();
            Rgb565::new(r >> 3, g >> 2, b >> 3)
        }
    }

    impl<const BITS: usize> ColorConvert<Rgb888> for Hub75Color<BITS> {
        fn from_color(color: Rgb888) -> Self {
            Self::from_rgb8(color.r(), color.g(), color.b())
        }

        fn to_color(self) -> Rgb888 {
            let (r, g, b) = self.to_rgb8();
            Rgb888::new(r, g, b)
        }
    }

    // Keep From/Into for backward compatibility
    impl<const BITS: usize> From<Rgb565> for Hub75Color<BITS> {
        fn from(color: Rgb565) -> Self {
            Self::from_color(color)
        }
    }

    impl<const BITS: usize> From<Rgb888> for Hub75Color<BITS> {
        fn from(color: Rgb888) -> Self {
            Self::from_color(color)
        }
    }

    impl<const BITS: usize> From<Hub75Color<BITS>> for Rgb565 {
        fn from(color: Hub75Color<BITS>) -> Self {
            color.to_color()
        }
    }

    impl<const BITS: usize> From<Hub75Color<BITS>> for Rgb888 {
        fn from(color: Hub75Color<BITS>) -> Self {
            color.to_color()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = Hub75Color::<6>::new(63, 32, 15);
        assert_eq!(color.r, 63);
        assert_eq!(color.g, 32);
        assert_eq!(color.b, 15);
    }

    #[test]
    fn test_color_clamping() {
        let color = Hub75Color::<4>::new(255, 255, 255);
        assert_eq!(color.r, 15); // 2^4 - 1 = 15
        assert_eq!(color.g, 15);
        assert_eq!(color.b, 15);
    }

    #[test]
    fn test_bit_extraction() {
        let color = Hub75Color::<4>::new(10, 5, 3); // 1010, 0101, 0011 in binary

        let (r0, g0, b0) = color.get_bit(0);
        assert_eq!((r0, g0, b0), (false, true, true)); // LSB

        let (r1, g1, b1) = color.get_bit(1);
        assert_eq!((r1, g1, b1), (true, false, true));

        let (r3, g3, b3) = color.get_bit(3);
        assert_eq!((r3, g3, b3), (true, false, false)); // MSB
    }

    #[test]
    fn test_rgb8_conversion() {
        let color = Hub75Color::<6>::from_rgb8(255, 128, 64);
        let (r, g, b) = color.to_rgb8();

        // Should be close to original values (some precision loss expected)
        assert!(r >= 252); // 63 << 2 = 252
        assert!(g >= 124 && g <= 128); // 32 << 2 = 128
        assert!(b >= 60 && b <= 64); // 16 << 2 = 64
    }
}
