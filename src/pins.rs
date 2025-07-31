//! Pin configuration and management for HUB75 displays

use embedded_hal::digital::OutputPin;
use crate::error::Hub75Error;

/// Complete pin configuration for a HUB75 display
pub struct Hub75Pins<P: OutputPin> {
    /// RGB pins for upper and lower halves
    pub rgb: Hub75RgbPins<P>,
    /// Address pins for row selection
    pub address: Hub75AddressPins<P>,
    /// Control pins for timing and latching
    pub control: Hub75ControlPins<P>,
}

/// RGB data pins for HUB75 interface
pub struct Hub75RgbPins<P: OutputPin> {
    /// Red pin for upper half of display
    pub r1: P,
    /// Green pin for upper half of display
    pub g1: P,
    /// Blue pin for upper half of display
    pub b1: P,
    /// Red pin for lower half of display
    pub r2: P,
    /// Green pin for lower half of display
    pub g2: P,
    /// Blue pin for lower half of display
    pub b2: P,
}

/// Address pins for row selection
pub struct Hub75AddressPins<P: OutputPin> {
    /// Address pin A (LSB)
    pub a: P,
    /// Address pin B
    pub b: P,
    /// Address pin C
    pub c: P,
    /// Address pin D (optional, for larger displays)
    pub d: Option<P>,
    /// Address pin E (optional, for very large displays)
    pub e: Option<P>,
}

/// Control pins for timing and data latching
pub struct Hub75ControlPins<P: OutputPin> {
    /// Clock pin for shifting data
    pub clk: P,
    /// Latch pin for transferring shift register to output
    pub lat: P,
    /// Output Enable pin (active low)
    pub oe: P,
}

impl<P: OutputPin> Hub75Pins<P> {
    /// Create a new pin configuration for a standard HUB75 display
    pub fn new(
        r1: P, g1: P, b1: P,
        r2: P, g2: P, b2: P,
        a: P, b: P, c: P, d: Option<P>, e: Option<P>,
        clk: P, lat: P, oe: P,
    ) -> Self {
        Self {
            rgb: Hub75RgbPins { r1, g1, b1, r2, g2, b2 },
            address: Hub75AddressPins { a, b, c, d, e },
            control: Hub75ControlPins { clk, lat, oe },
        }
    }

    /// Create pin configuration for 32x16 display (3 address pins)
    pub fn new_32x16(
        r1: P, g1: P, b1: P,
        r2: P, g2: P, b2: P,
        a: P, b: P, c: P,
        clk: P, lat: P, oe: P,
    ) -> Self {
        Self::new(r1, g1, b1, r2, g2, b2, a, b, c, None, None, clk, lat, oe)
    }

    /// Create pin configuration for 64x32 display (4 address pins)
    pub fn new_64x32(
        r1: P, g1: P, b1: P,
        r2: P, g2: P, b2: P,
        a: P, b: P, c: P, d: P,
        clk: P, lat: P, oe: P,
    ) -> Self {
        Self::new(r1, g1, b1, r2, g2, b2, a, b, c, Some(d), None, clk, lat, oe)
    }

    /// Create pin configuration for 64x64 display (5 address pins)
    pub fn new_64x64(
        r1: P, g1: P, b1: P,
        r2: P, g2: P, b2: P,
        a: P, b: P, c: P, d: P, e: P,
        clk: P, lat: P, oe: P,
    ) -> Self {
        Self::new(r1, g1, b1, r2, g2, b2, a, b, c, Some(d), Some(e), clk, lat, oe)
    }

    /// Initialize all pins to their default states
    pub fn init(&mut self) -> Result<(), Hub75Error> {
        // Initialize RGB pins to low
        self.rgb.r1.set_low().map_err(|_| Hub75Error::PinError)?;
        self.rgb.g1.set_low().map_err(|_| Hub75Error::PinError)?;
        self.rgb.b1.set_low().map_err(|_| Hub75Error::PinError)?;
        self.rgb.r2.set_low().map_err(|_| Hub75Error::PinError)?;
        self.rgb.g2.set_low().map_err(|_| Hub75Error::PinError)?;
        self.rgb.b2.set_low().map_err(|_| Hub75Error::PinError)?;

        // Initialize address pins to low
        self.address.a.set_low().map_err(|_| Hub75Error::PinError)?;
        self.address.b.set_low().map_err(|_| Hub75Error::PinError)?;
        self.address.c.set_low().map_err(|_| Hub75Error::PinError)?;
        if let Some(ref mut d) = self.address.d {
            d.set_low().map_err(|_| Hub75Error::PinError)?;
        }
        if let Some(ref mut e) = self.address.e {
            e.set_low().map_err(|_| Hub75Error::PinError)?;
        }

        // Initialize control pins
        self.control.clk.set_low().map_err(|_| Hub75Error::PinError)?;
        self.control.lat.set_low().map_err(|_| Hub75Error::PinError)?;
        self.control.oe.set_high().map_err(|_| Hub75Error::PinError)?; // OE is active low

        Ok(())
    }

    /// Get the number of address pins available
    pub fn address_pin_count(&self) -> usize {
        let mut count = 3; // A, B, C are always present
        if self.address.d.is_some() {
            count += 1;
        }
        if self.address.e.is_some() {
            count += 1;
        }
        count
    }

    /// Get the maximum number of rows that can be addressed
    pub fn max_addressable_rows(&self) -> usize {
        1 << self.address_pin_count()
    }
}

impl<P: OutputPin> Hub75RgbPins<P> {
    /// Set RGB values for both upper and lower halves
    pub fn set_rgb(&mut self, 
                   upper_r: bool, upper_g: bool, upper_b: bool,
                   lower_r: bool, lower_g: bool, lower_b: bool) -> Result<(), Hub75Error> {
        // Set upper half RGB
        if upper_r {
            self.r1.set_high().map_err(|_| Hub75Error::PinError)?;
        } else {
            self.r1.set_low().map_err(|_| Hub75Error::PinError)?;
        }
        
        if upper_g {
            self.g1.set_high().map_err(|_| Hub75Error::PinError)?;
        } else {
            self.g1.set_low().map_err(|_| Hub75Error::PinError)?;
        }
        
        if upper_b {
            self.b1.set_high().map_err(|_| Hub75Error::PinError)?;
        } else {
            self.b1.set_low().map_err(|_| Hub75Error::PinError)?;
        }

        // Set lower half RGB
        if lower_r {
            self.r2.set_high().map_err(|_| Hub75Error::PinError)?;
        } else {
            self.r2.set_low().map_err(|_| Hub75Error::PinError)?;
        }
        
        if lower_g {
            self.g2.set_high().map_err(|_| Hub75Error::PinError)?;
        } else {
            self.g2.set_low().map_err(|_| Hub75Error::PinError)?;
        }
        
        if lower_b {
            self.b2.set_high().map_err(|_| Hub75Error::PinError)?;
        } else {
            self.b2.set_low().map_err(|_| Hub75Error::PinError)?;
        }

        Ok(())
    }

    /// Clear all RGB pins (set to low)
    pub fn clear(&mut self) -> Result<(), Hub75Error> {
        self.set_rgb(false, false, false, false, false, false)
    }
}

impl<P: OutputPin> Hub75AddressPins<P> {
    /// Set the address pins to select a specific row
    pub fn set_address(&mut self, row: usize) -> Result<(), Hub75Error> {
        // Set pin A (bit 0)
        if (row & 0x01) != 0 {
            self.a.set_high().map_err(|_| Hub75Error::PinError)?;
        } else {
            self.a.set_low().map_err(|_| Hub75Error::PinError)?;
        }

        // Set pin B (bit 1)
        if (row & 0x02) != 0 {
            self.b.set_high().map_err(|_| Hub75Error::PinError)?;
        } else {
            self.b.set_low().map_err(|_| Hub75Error::PinError)?;
        }

        // Set pin C (bit 2)
        if (row & 0x04) != 0 {
            self.c.set_high().map_err(|_| Hub75Error::PinError)?;
        } else {
            self.c.set_low().map_err(|_| Hub75Error::PinError)?;
        }

        // Set pin D (bit 3) if available
        if let Some(ref mut d) = self.d {
            if (row & 0x08) != 0 {
                d.set_high().map_err(|_| Hub75Error::PinError)?;
            } else {
                d.set_low().map_err(|_| Hub75Error::PinError)?;
            }
        }

        // Set pin E (bit 4) if available
        if let Some(ref mut e) = self.e {
            if (row & 0x10) != 0 {
                e.set_high().map_err(|_| Hub75Error::PinError)?;
            } else {
                e.set_low().map_err(|_| Hub75Error::PinError)?;
            }
        }

        Ok(())
    }
}

impl<P: OutputPin> Hub75ControlPins<P> {
    /// Generate a clock pulse
    pub fn clock_pulse(&mut self) -> Result<(), Hub75Error> {
        self.clk.set_high().map_err(|_| Hub75Error::PinError)?;
        self.clk.set_low().map_err(|_| Hub75Error::PinError)?;
        Ok(())
    }

    /// Generate a latch pulse
    pub fn latch_pulse(&mut self) -> Result<(), Hub75Error> {
        self.lat.set_high().map_err(|_| Hub75Error::PinError)?;
        self.lat.set_low().map_err(|_| Hub75Error::PinError)?;
        Ok(())
    }

    /// Enable output (set OE low)
    pub fn enable_output(&mut self) -> Result<(), Hub75Error> {
        self.oe.set_low().map_err(|_| Hub75Error::PinError)
    }

    /// Disable output (set OE high)
    pub fn disable_output(&mut self) -> Result<(), Hub75Error> {
        self.oe.set_high().map_err(|_| Hub75Error::PinError)
    }
}