//! Core HUB75 display driver implementation

use crate::{
    color::Hub75Color, frame_buffer::Hub75FrameBuffer, pins::Hub75Pins, Hub75Error,
};
use embassy_time::{Duration, Instant, Timer};
use embedded_hal::digital::OutputPin;

/// Brightness levels for the display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Brightness {
    level: u8,
}

impl Brightness {
    /// Maximum brightness level
    pub const MAX: Self = Self { level: 255 };
    /// Minimum brightness level
    pub const MIN: Self = Self { level: 0 };

    /// Create a new brightness level (0-255)
    pub fn new(level: u8) -> Self {
        Self { level }
    }

    /// Get the brightness level
    pub fn level(&self) -> u8 {
        self.level
    }
}

impl Default for Brightness {
    fn default() -> Self {
        Self { level: 128 } // 50% brightness
    }
}

impl core::ops::Add<u8> for Brightness {
    type Output = Self;

    fn add(self, rhs: u8) -> Self::Output {
        Self {
            level: self.level.saturating_add(rhs),
        }
    }
}

impl core::ops::Sub<u8> for Brightness {
    type Output = Self;

    fn sub(self, rhs: u8) -> Self::Output {
        Self {
            level: self.level.saturating_sub(rhs),
        }
    }
}

/// Main HUB75 display driver
pub struct Hub75Display<
    P: OutputPin + 'static,
    const WIDTH: usize,
    const HEIGHT: usize,
    const COLOR_BITS: usize,
> {
    /// Pin configuration
    pins: Hub75Pins<P>,
    /// Front frame buffer (currently being displayed)
    front_buffer: Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>,
    /// Back frame buffer (for double buffering)
    back_buffer: Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>,
    /// Current row being scanned
    current_row: usize,
    /// Current bit plane being displayed
    current_bit_plane: usize,
    /// Display brightness
    brightness: Brightness,
    /// Base refresh interval
    refresh_interval: Duration,
    /// Whether double buffering is enabled
    double_buffering: bool,
}

impl<P, const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize>
    Hub75Display<P, WIDTH, HEIGHT, COLOR_BITS>
where
    P: OutputPin,
{
    /// Create a new HUB75 display driver
    pub fn new(mut pins: Hub75Pins<P>) -> Result<Self, Hub75Error> {
        // Initialize pins to default state
        pins.init()?;

        // Validate display dimensions
        let max_rows = pins.max_addressable_rows();
        if HEIGHT / 2 > max_rows {
            return Err(Hub75Error::InvalidCoordinates);
        }

        Ok(Self {
            pins,
            front_buffer: Hub75FrameBuffer::new(),
            back_buffer: Hub75FrameBuffer::new(),
            current_row: 0,
            current_bit_plane: 0,
            brightness: Brightness::default(),
            refresh_interval: Duration::from_micros(100), // 10kHz base refresh rate
            double_buffering: false,
        })
    }

    /// Enable or disable double buffering
    pub fn set_double_buffering(&mut self, enabled: bool) {
        self.double_buffering = enabled;
    }

    /// Swap front and back buffers (for double buffering)
    pub fn swap_buffers(&mut self) {
        if self.double_buffering {
            self.front_buffer.swap(&mut self.back_buffer);
        }
    }

    /// Get a reference to the back buffer for drawing
    pub fn back_buffer(&mut self) -> &mut Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS> {
        if self.double_buffering {
            &mut self.back_buffer
        } else {
            &mut self.front_buffer
        }
    }

    /// Get a reference to the front buffer (currently displayed)
    pub fn front_buffer(&self) -> &Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS> {
        &self.front_buffer
    }

    /// Set the display brightness
    pub fn set_brightness(&mut self, brightness: Brightness) {
        self.brightness = brightness;
    }

    /// Get the current brightness
    pub fn brightness(&self) -> Brightness {
        self.brightness
    }

    /// Set the base refresh interval
    pub fn set_refresh_interval(&mut self, interval: Duration) {
        self.refresh_interval = interval;
    }

    /// Clear the display (set all pixels to black)
    pub fn clear(&mut self) {
        self.back_buffer().clear();
        if !self.double_buffering {
            self.front_buffer.clear();
        }
    }

    /// Set a pixel at the specified coordinates
    pub fn set_pixel(
        &mut self,
        x: usize,
        y: usize,
        color: Hub75Color<COLOR_BITS>,
    ) -> Result<(), Hub75Error> {
        self.back_buffer().set_pixel(x, y, color)
    }

    /// Get a pixel at the specified coordinates
    pub fn get_pixel(&self, x: usize, y: usize) -> Result<Hub75Color<COLOR_BITS>, Hub75Error> {
        self.front_buffer().get_pixel(x, y)
    }

    /// Fill the display with a single color
    pub fn fill(&mut self, color: Hub75Color<COLOR_BITS>) {
        self.back_buffer().fill(color);
    }

    /// Render a single bit plane for the current row
    pub fn render_bit_plane(&mut self) -> Result<(), Hub75Error> {
        // Disable output during data loading
        self.pins.control.disable_output()?;

        // Set row address
        self.pins.address.set_address(self.current_row)?;

        // Get bit plane data for current row
        let bit_data = self
            .front_buffer
            .get_row_bit_plane(self.current_row, self.current_bit_plane)?;

        // Shift out RGB data for all columns
        for &(upper_r, upper_g, upper_b, lower_r, lower_g, lower_b) in &bit_data {
            // Set RGB pins
            self.pins
                .rgb
                .set_rgb(upper_r, upper_g, upper_b, lower_r, lower_g, lower_b)?;

            // Clock pulse to shift data
            self.pins.control.clock_pulse()?;
        }

        // Latch the data
        self.pins.control.latch_pulse()?;

        // Enable output
        self.pins.control.enable_output()?;

        Ok(())
    }

    /// Render a complete frame using Binary Code Modulation
    pub async fn render_frame(&mut self) -> Result<(), Hub75Error> {
        for bit_plane in 0..COLOR_BITS {
            for row in 0..(HEIGHT / 2) {
                self.current_row = row;
                self.current_bit_plane = bit_plane;

                self.render_bit_plane()?;

                // BCM timing - exponentially longer delays for higher bit planes
                let bit_duration = self.refresh_interval * (1 << bit_plane);

                // Apply brightness scaling
                let brightness_factor = self.brightness.level() as u64;
                let scaled_duration =
                    Duration::from_micros(bit_duration.as_micros() * brightness_factor / 255);

                Timer::after(scaled_duration).await;

                // Disable output before moving to next row/bit plane
                self.pins.control.disable_output().ok();
            }
        }

        Ok(())
    }

    /// Continuous refresh task for embassy
    pub async fn refresh_task(&mut self) -> ! {
        loop {
            if self.render_frame().await.is_err() {
                // Handle error - maybe reset pins or continue
                Timer::after(Duration::from_millis(1)).await;
            }
        }
    }

    /// Display a frame for a specific duration
    pub async fn display_frame(
        &mut self,
        frame: Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>,
        duration: Duration,
    ) -> Result<(), Hub75Error> {
        // Copy frame to appropriate buffer
        if self.double_buffering {
            self.back_buffer.copy_from(&frame);
            self.swap_buffers();
        } else {
            self.front_buffer.copy_from(&frame);
        }

        let end_time = Instant::now() + duration;

        while Instant::now() < end_time {
            self.render_frame().await?;
        }

        Ok(())
    }

    /// Get display dimensions
    pub const fn dimensions(&self) -> (usize, usize) {
        (WIDTH, HEIGHT)
    }

    /// Get color bit depth
    pub const fn color_bits(&self) -> usize {
        COLOR_BITS
    }

    /// Get the number of addressable rows (HEIGHT / 2)
    pub const fn addressable_rows(&self) -> usize {
        HEIGHT / 2
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

    impl<P, const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize> DrawTarget
        for Hub75Display<P, WIDTH, HEIGHT, COLOR_BITS>
    where
        P: OutputPin,
    {
        type Color = Rgb565;
        type Error = Hub75Error;

        fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
        where
            I: IntoIterator<Item = Pixel<Self::Color>>,
        {
            self.back_buffer().draw_iter(pixels)
        }
    }

    impl<P, const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize> OriginDimensions
        for Hub75Display<P, WIDTH, HEIGHT, COLOR_BITS>
    where
        P: OutputPin,
    {
        fn size(&self) -> Size {
            Size::new(WIDTH as u32, HEIGHT as u32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal::digital::{ErrorType, OutputPin};

    // Mock pin for testing
    struct MockPin {
        state: bool,
    }

    impl MockPin {
        fn new() -> Self {
            Self { state: false }
        }
    }

    impl ErrorType for MockPin {
        type Error = ();
    }

    impl OutputPin for MockPin {
        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.state = false;
            Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.state = true;
            Ok(())
        }
    }

    #[test]
    fn test_display_creation() {
        let pins = Hub75Pins::new_64x32(
            MockPin::new(),
            MockPin::new(),
            MockPin::new(), // RGB1
            MockPin::new(),
            MockPin::new(),
            MockPin::new(), // RGB2
            MockPin::new(),
            MockPin::new(),
            MockPin::new(),
            MockPin::new(), // Address
            MockPin::new(),
            MockPin::new(),
            MockPin::new(), // Control
        );

        let display = Hub75Display::<_, 64, 32, 6>::new(pins);
        assert!(display.is_ok());

        let display = display.unwrap();
        assert_eq!(display.dimensions(), (64, 32));
        assert_eq!(display.color_bits(), 6);
        assert_eq!(display.addressable_rows(), 16);
    }

    #[test]
    fn test_brightness_operations() {
        let mut brightness = Brightness::new(100);
        assert_eq!(brightness.level(), 100);

        brightness = brightness + 50;
        assert_eq!(brightness.level(), 150);

        brightness = brightness - 25;
        assert_eq!(brightness.level(), 125);

        // Test saturation
        brightness = Brightness::new(250) + 20;
        assert_eq!(brightness.level(), 255);

        brightness = Brightness::new(10) - 20;
        assert_eq!(brightness.level(), 0);
    }
}
