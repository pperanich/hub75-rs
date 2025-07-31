//! Frame buffer management for HUB75 displays

use crate::{color::Hub75Color, Hub75Error};
use heapless::Vec;

/// Frame buffer for storing pixel data
#[derive(Debug, PartialEq, Eq)]
pub struct Hub75FrameBuffer<const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize> {
    /// Pixel data stored as a flat array
    pixels: [[Hub75Color<COLOR_BITS>; WIDTH]; HEIGHT],
}

impl<const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize>
    Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>
{
    /// Create a new frame buffer filled with black pixels
    pub fn new() -> Self {
        Self {
            pixels: [[Hub75Color::black(); WIDTH]; HEIGHT],
        }
    }

    /// Clear the frame buffer (set all pixels to black)
    pub fn clear(&mut self) {
        self.fill(Hub75Color::black());
    }

    /// Fill the entire frame buffer with a single color
    pub fn fill(&mut self, color: Hub75Color<COLOR_BITS>) {
        for row in &mut self.pixels {
            for pixel in row {
                *pixel = color;
            }
        }
    }

    /// Get a mutable reference to a pixel at the specified coordinates
    #[inline(always)]
    pub fn pixel_mut(
        &mut self,
        x: usize,
        y: usize,
    ) -> Result<&mut Hub75Color<COLOR_BITS>, Hub75Error> {
        if x >= WIDTH || y >= HEIGHT {
            Err(Hub75Error::InvalidCoordinates)
        } else {
            Ok(unsafe { self.pixels.get_unchecked_mut(y).get_unchecked_mut(x) })
        }
    }

    /// Get a reference to a pixel at the specified coordinates
    #[inline(always)]
    pub fn pixel(&self, x: usize, y: usize) -> Result<&Hub75Color<COLOR_BITS>, Hub75Error> {
        if x >= WIDTH || y >= HEIGHT {
            Err(Hub75Error::InvalidCoordinates)
        } else {
            Ok(unsafe { self.pixels.get_unchecked(y).get_unchecked(x) })
        }
    }

    /// Set a pixel at the specified coordinates
    #[inline(always)]
    pub fn set_pixel(
        &mut self,
        x: usize,
        y: usize,
        color: Hub75Color<COLOR_BITS>,
    ) -> Result<(), Hub75Error> {
        *self.pixel_mut(x, y)? = color;
        Ok(())
    }

    /// Get a pixel at the specified coordinates
    #[inline(always)]
    pub fn get_pixel(&self, x: usize, y: usize) -> Result<Hub75Color<COLOR_BITS>, Hub75Error> {
        Ok(*self.pixel(x, y)?)
    }

    /// Get a pixel at the specified coordinates without bounds checking
    ///
    /// # Safety
    /// The caller must ensure that x < WIDTH and y < HEIGHT
    #[inline(always)]
    pub unsafe fn get_pixel_unchecked(&self, x: usize, y: usize) -> Hub75Color<COLOR_BITS> {
        *self.pixels.get_unchecked(y).get_unchecked(x)
    }

    /// Set a pixel at the specified coordinates without bounds checking
    ///
    /// # Safety
    /// The caller must ensure that x < WIDTH and y < HEIGHT
    #[inline(always)]
    pub unsafe fn set_pixel_unchecked(
        &mut self,
        x: usize,
        y: usize,
        color: Hub75Color<COLOR_BITS>,
    ) {
        *self.pixels.get_unchecked_mut(y).get_unchecked_mut(x) = color;
    }

    /// Get the width of the frame buffer
    pub const fn width(&self) -> usize {
        WIDTH
    }

    /// Get the height of the frame buffer
    pub const fn height(&self) -> usize {
        HEIGHT
    }

    /// Get the color bit depth
    pub const fn color_bits(&self) -> usize {
        COLOR_BITS
    }

    /// Get a row of pixels for efficient scanning
    pub fn get_row(&self, y: usize) -> Result<&[Hub75Color<COLOR_BITS>; WIDTH], Hub75Error> {
        if y >= HEIGHT {
            return Err(Hub75Error::InvalidCoordinates);
        }

        Ok(&self.pixels[y])
    }

    /// Get a mutable row of pixels for efficient modification
    pub fn get_row_mut(
        &mut self,
        y: usize,
    ) -> Result<&mut [Hub75Color<COLOR_BITS>; WIDTH], Hub75Error> {
        if y >= HEIGHT {
            return Err(Hub75Error::InvalidCoordinates);
        }

        Ok(&mut self.pixels[y])
    }

    /// Copy data from another frame buffer
    pub fn copy_from(&mut self, other: &Self) {
        self.pixels.copy_from_slice(&other.pixels);
    }

    /// Swap the contents of this frame buffer with another
    pub fn swap(&mut self, other: &mut Self) {
        core::mem::swap(&mut self.pixels, &mut other.pixels);
    }

    /// Get RGB bit values for a specific row and bit plane
    /// Returns vectors of (upper_r, upper_g, upper_b, lower_r, lower_g, lower_b) for each column
    pub fn get_row_bit_plane(
        &self,
        row: usize,
        bit_plane: usize,
    ) -> Result<Vec<(bool, bool, bool, bool, bool, bool), WIDTH>, Hub75Error> {
        if row >= HEIGHT / 2 {
            return Err(Hub75Error::InvalidCoordinates);
        }

        if bit_plane >= COLOR_BITS {
            return Err(Hub75Error::InvalidColor);
        }

        let mut result = Vec::new();

        for x in 0..WIDTH {
            let upper_pixel = self.pixels[row][x];
            let lower_pixel = self.pixels[row + HEIGHT / 2][x];

            let (upper_r, upper_g, upper_b) = upper_pixel.get_bit(bit_plane);
            let (lower_r, lower_g, lower_b) = lower_pixel.get_bit(bit_plane);

            result
                .push((upper_r, upper_g, upper_b, lower_r, lower_g, lower_b))
                .map_err(|_| Hub75Error::BufferOverflow)?;
        }

        Ok(result)
    }

    /// Create a frame buffer from raw RGB data
    pub fn from_rgb_data(data: &[u8]) -> Result<Self, Hub75Error> {
        if data.len() != WIDTH * HEIGHT * 3 {
            return Err(Hub75Error::InvalidColor);
        }

        let mut buffer = Self::new();

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let idx = (y * WIDTH + x) * 3;
                let color = Hub75Color::from_rgb8(data[idx], data[idx + 1], data[idx + 2]);
                buffer.set_pixel(x, y, color)?;
            }
        }

        Ok(buffer)
    }

    /// Convert frame buffer to raw RGB data
    pub fn to_rgb_data(&self) -> heapless::Vec<u8, 65536> {
        let mut data = heapless::Vec::new();

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let color = self.pixels[y][x];
                let (r, g, b) = color.to_rgb8();
                data.push(r).ok();
                data.push(g).ok();
                data.push(b).ok();
            }
        }

        data
    }
}

impl<const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize> Default
    for Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize> Clone
    for Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>
{
    fn clone(&self) -> Self {
        Self {
            pixels: self.pixels,
        }
    }
}

#[cfg(feature = "embedded-graphics")]
mod embedded_graphics_support {
    use super::*;
    use embedded_graphics_core::{
        draw_target::DrawTarget,
        geometry::{OriginDimensions, Size},
        pixelcolor::Rgb565,
        Pixel,
    };

    impl<const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize> DrawTarget
        for Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>
    {
        type Color = Rgb565;
        type Error = Hub75Error;

        fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Pixel<Self::Color>>,
        {
            for Pixel(coord, color) in pixels {
                let x = coord.x as usize;
                let y = coord.y as usize;
                if x < WIDTH && y < HEIGHT {
                    let hub75_color = Hub75Color::from(color);
                    self.set_pixel(x, y, hub75_color)?;
                }
            }
            Ok(())
        }
    }

    impl<const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize> OriginDimensions
        for Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>
    {
        fn size(&self) -> Size {
            Size::new(WIDTH as u32, HEIGHT as u32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_buffer_creation() {
        let buffer = Hub75FrameBuffer::<64, 32, 6>::new();
        assert_eq!(buffer.width(), 64);
        assert_eq!(buffer.height(), 32);
        assert_eq!(buffer.color_bits(), 6);
    }

    #[test]
    fn test_pixel_operations() {
        let mut buffer = Hub75FrameBuffer::<64, 32, 6>::new();
        let red = Hub75Color::red();

        buffer.set_pixel(10, 15, red).unwrap();
        let pixel = buffer.get_pixel(10, 15).unwrap();
        assert_eq!(pixel, red);
    }

    #[test]
    fn test_bounds_checking() {
        let mut buffer = Hub75FrameBuffer::<64, 32, 6>::new();
        let red = Hub75Color::red();

        assert!(buffer.set_pixel(64, 15, red).is_err()); // x out of bounds
        assert!(buffer.set_pixel(10, 32, red).is_err()); // y out of bounds
        assert!(buffer.get_pixel(64, 15).is_err());
        assert!(buffer.get_pixel(10, 32).is_err());
    }

    #[test]
    fn test_fill_and_clear() {
        let mut buffer = Hub75FrameBuffer::<64, 32, 6>::new();
        let blue = Hub75Color::blue();

        buffer.fill(blue);
        assert_eq!(buffer.get_pixel(0, 0).unwrap(), blue);
        assert_eq!(buffer.get_pixel(63, 31).unwrap(), blue);

        buffer.clear();
        assert_eq!(buffer.get_pixel(0, 0).unwrap(), Hub75Color::black());
        assert_eq!(buffer.get_pixel(63, 31).unwrap(), Hub75Color::black());
    }
}
