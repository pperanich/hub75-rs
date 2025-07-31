//! Pin configuration and management for HUB75 displays
//!
//! This module provides structures for organizing and managing the various pins
//! required for HUB75 LED matrix displays. The pins are grouped by function:
//!
//! - **RGB pins**: Data lines for color information (upper and lower halves)
//! - **Address pins**: Row selection lines (A, B, C, D, E)
//! - **Control pins**: Timing and synchronization (CLK, LAT, OE)
//!
//! # HUB75 Pin Layout
//!
//! A typical HUB75 connector has the following pins:
//!
//! ```text
//! R1  G1  B1  GND
//! R2  G2  B2  GND  
//! A   B   C   D
//! CLK LAT OE  GND
//! ```
//!
//! # Examples
//!
//! ```rust,no_run
//! use hub75::{Hub75Pins, Hub75RgbPins, Hub75AddressPins, Hub75ControlPins};
//! use embedded_hal::digital::OutputPin;
//!
//! # fn example(pin: impl OutputPin + Clone) {
//! let pins = Hub75Pins {
//!     rgb: Hub75RgbPins {
//!         r1: pin.clone(), g1: pin.clone(), b1: pin.clone(),
//!         r2: pin.clone(), g2: pin.clone(), b2: pin.clone(),
//!     },
//!     address: Hub75AddressPins {
//!         a: pin.clone(), b: pin.clone(), c: pin.clone(),
//!         d: Some(pin.clone()), e: None, // D pin optional, E pin not used
//!     },
//!     control: Hub75ControlPins {
//!         clk: pin.clone(), lat: pin.clone(), oe: pin,
//!     },
//! };
//! # }
//! ```

use crate::{pin_op, Hub75Error};
use embedded_hal::digital::OutputPin;

/// Complete pin configuration for a HUB75 display
///
/// This structure organizes all the pins required for a HUB75 display into
/// logical groups. It provides a clean interface for pin management and
/// ensures all required pins are properly configured.
///
/// # Pin Requirements
///
/// - **6 RGB pins**: R1, G1, B1, R2, G2, B2 (always required)
/// - **3-5 address pins**: A, B, C (required), D, E (optional, depends on panel size)
/// - **3 control pins**: CLK, LAT, OE (always required)
///
/// Total: 12-16 pins depending on panel size
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

/// Builder for constructing Hub75Pins with a fluent interface
pub struct Hub75PinsBuilder<P: OutputPin> {
    rgb: Option<(P, P, P, P, P, P)>,
    address: Option<(P, P, P, Option<P>, Option<P>)>,
    control: Option<(P, P, P)>,
}

impl<P: OutputPin> Hub75Pins<P> {
    /// Create a builder for constructing pin configuration
    pub fn builder() -> Hub75PinsBuilder<P> {
        Hub75PinsBuilder {
            rgb: None,
            address: None,
            control: None,
        }
    }

    /// Create a new pin configuration for a standard HUB75 display
    pub fn new(
        r1: P,
        g1: P,
        b1: P,
        r2: P,
        g2: P,
        b2: P,
        a: P,
        b: P,
        c: P,
        d: Option<P>,
        e: Option<P>,
        clk: P,
        lat: P,
        oe: P,
    ) -> Self {
        Self {
            rgb: Hub75RgbPins {
                r1,
                g1,
                b1,
                r2,
                g2,
                b2,
            },
            address: Hub75AddressPins { a, b, c, d, e },
            control: Hub75ControlPins { clk, lat, oe },
        }
    }

    /// Create pin configuration for 32x16 display (3 address pins)
    pub fn new_32x16(
        r1: P,
        g1: P,
        b1: P,
        r2: P,
        g2: P,
        b2: P,
        a: P,
        b: P,
        c: P,
        clk: P,
        lat: P,
        oe: P,
    ) -> Self {
        Self::new(r1, g1, b1, r2, g2, b2, a, b, c, None, None, clk, lat, oe)
    }

    /// Create pin configuration for 64x32 display (4 address pins)
    pub fn new_64x32(
        r1: P,
        g1: P,
        b1: P,
        r2: P,
        g2: P,
        b2: P,
        a: P,
        b: P,
        c: P,
        d: P,
        clk: P,
        lat: P,
        oe: P,
    ) -> Self {
        Self::new(r1, g1, b1, r2, g2, b2, a, b, c, Some(d), None, clk, lat, oe)
    }

    /// Create pin configuration for 64x64 display (5 address pins)
    pub fn new_64x64(
        r1: P,
        g1: P,
        b1: P,
        r2: P,
        g2: P,
        b2: P,
        a: P,
        b: P,
        c: P,
        d: P,
        e: P,
        clk: P,
        lat: P,
        oe: P,
    ) -> Self {
        Self::new(
            r1,
            g1,
            b1,
            r2,
            g2,
            b2,
            a,
            b,
            c,
            Some(d),
            Some(e),
            clk,
            lat,
            oe,
        )
    }

    /// Initialize all pins to their default states
    pub fn init(&mut self) -> Result<(), Hub75Error> {
        // Initialize RGB pins to low
        pin_op!(self.rgb.r1.set_low());
        pin_op!(self.rgb.g1.set_low());
        pin_op!(self.rgb.b1.set_low());
        pin_op!(self.rgb.r2.set_low());
        pin_op!(self.rgb.g2.set_low());
        pin_op!(self.rgb.b2.set_low());

        // Initialize address pins to low
        pin_op!(self.address.a.set_low());
        pin_op!(self.address.b.set_low());
        pin_op!(self.address.c.set_low());
        if let Some(ref mut d) = self.address.d {
            pin_op!(d.set_low());
        }
        if let Some(ref mut e) = self.address.e {
            pin_op!(e.set_low());
        }

        // Initialize control pins
        pin_op!(self.control.clk.set_low());
        pin_op!(self.control.lat.set_low());
        pin_op!(self.control.oe.set_high()); // OE is active low

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

impl<P: OutputPin> Hub75PinsBuilder<P> {
    /// Set RGB pins for upper and lower halves
    pub fn rgb(mut self, r1: P, g1: P, b1: P, r2: P, g2: P, b2: P) -> Self {
        self.rgb = Some((r1, g1, b1, r2, g2, b2));
        self
    }

    /// Set address pins (A, B, C are required)
    pub fn address(mut self, a: P, b: P, c: P) -> Self {
        self.address = Some((a, b, c, None, None));
        self
    }

    /// Set address pins with optional D and E pins
    pub fn address_with_optional(mut self, a: P, b: P, c: P, d: Option<P>, e: Option<P>) -> Self {
        self.address = Some((a, b, c, d, e));
        self
    }

    /// Set control pins
    pub fn control(mut self, clk: P, lat: P, oe: P) -> Self {
        self.control = Some((clk, lat, oe));
        self
    }

    /// Build the Hub75Pins configuration
    pub fn build(self) -> Result<Hub75Pins<P>, Hub75Error> {
        let rgb = self.rgb.ok_or(Hub75Error::InvalidCoordinates)?;
        let address = self.address.ok_or(Hub75Error::InvalidCoordinates)?;
        let control = self.control.ok_or(Hub75Error::InvalidCoordinates)?;

        Ok(Hub75Pins {
            rgb: Hub75RgbPins {
                r1: rgb.0,
                g1: rgb.1,
                b1: rgb.2,
                r2: rgb.3,
                g2: rgb.4,
                b2: rgb.5,
            },
            address: Hub75AddressPins {
                a: address.0,
                b: address.1,
                c: address.2,
                d: address.3,
                e: address.4,
            },
            control: Hub75ControlPins {
                clk: control.0,
                lat: control.1,
                oe: control.2,
            },
        })
    }
}

impl<P: OutputPin> Hub75RgbPins<P> {
    /// Set RGB values for both upper and lower halves
    pub fn set_rgb(
        &mut self,
        upper_r: bool,
        upper_g: bool,
        upper_b: bool,
        lower_r: bool,
        lower_g: bool,
        lower_b: bool,
    ) -> Result<(), Hub75Error> {
        // Helper macro to set pin state
        macro_rules! set_pin {
            ($pin:expr, $state:expr) => {
                if $state {
                    pin_op!($pin.set_high())
                } else {
                    pin_op!($pin.set_low())
                }
            };
        }

        // Set all pins in one go
        set_pin!(self.r1, upper_r);
        set_pin!(self.g1, upper_g);
        set_pin!(self.b1, upper_b);
        set_pin!(self.r2, lower_r);
        set_pin!(self.g2, lower_g);
        set_pin!(self.b2, lower_b);

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
        // Helper macro to set address pin based on bit
        macro_rules! set_addr_pin {
            ($pin:expr, $bit:expr) => {
                if (row & (1 << $bit)) != 0 {
                    pin_op!($pin.set_high())
                } else {
                    pin_op!($pin.set_low())
                }
            };
        }

        // Set required address pins
        set_addr_pin!(self.a, 0);
        set_addr_pin!(self.b, 1);
        set_addr_pin!(self.c, 2);

        // Set optional address pins
        if let Some(ref mut d) = self.d {
            set_addr_pin!(d, 3);
        }
        if let Some(ref mut e) = self.e {
            set_addr_pin!(e, 4);
        }

        Ok(())
    }
}

impl<P: OutputPin> Hub75ControlPins<P> {
    /// Generate a clock pulse
    pub fn clock_pulse(&mut self) -> Result<(), Hub75Error> {
        pin_op!(self.clk.set_high());
        pin_op!(self.clk.set_low());
        Ok(())
    }

    /// Generate a latch pulse
    pub fn latch_pulse(&mut self) -> Result<(), Hub75Error> {
        pin_op!(self.lat.set_high());
        pin_op!(self.lat.set_low());
        Ok(())
    }

    /// Enable output (set OE low)
    pub fn enable_output(&mut self) -> Result<(), Hub75Error> {
        pin_op!(self.oe.set_low());
        Ok(())
    }

    /// Disable output (set OE high)
    pub fn disable_output(&mut self) -> Result<(), Hub75Error> {
        pin_op!(self.oe.set_high());
        Ok(())
    }
}
