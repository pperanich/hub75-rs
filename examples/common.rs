//! Common utilities shared between examples

use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::*;
use hub75_embassy::Hub75Pins;

/// Standard pin configuration for RP2040 with 64x32 HUB75 display
pub fn setup_64x32_pins() -> Hub75Pins<Output<'static>> {
    let p = embassy_rp::init(Default::default());

    Hub75Pins::new_64x32(
        // RGB pins for upper half (R1, G1, B1)
        Output::new(p.PIN_2, Level::Low), // R1
        Output::new(p.PIN_3, Level::Low), // G1
        Output::new(p.PIN_4, Level::Low), // B1
        // RGB pins for lower half (R2, G2, B2)
        Output::new(p.PIN_5, Level::Low), // R2
        Output::new(p.PIN_6, Level::Low), // G2
        Output::new(p.PIN_7, Level::Low), // B2
        // Address pins (A, B, C, D)
        Output::new(p.PIN_8, Level::Low),  // A
        Output::new(p.PIN_9, Level::Low),  // B
        Output::new(p.PIN_10, Level::Low), // C
        Output::new(p.PIN_11, Level::Low), // D
        // Control pins (CLK, LAT, OE)
        Output::new(p.PIN_12, Level::Low),  // CLK
        Output::new(p.PIN_13, Level::Low),  // LAT
        Output::new(p.PIN_14, Level::High), // OE (active low, so start high)
    )
}

/// Alternative pin configuration using builder pattern
pub fn setup_64x32_pins_builder() -> Result<Hub75Pins<Output<'static>>, hub75_embassy::Hub75Error> {
    let p = embassy_rp::init(Default::default());

    Hub75Pins::builder()
        .rgb(
            Output::new(p.PIN_2, Level::Low), // R1
            Output::new(p.PIN_3, Level::Low), // G1
            Output::new(p.PIN_4, Level::Low), // B1
            Output::new(p.PIN_5, Level::Low), // R2
            Output::new(p.PIN_6, Level::Low), // G2
            Output::new(p.PIN_7, Level::Low), // B2
        )
        .address_with_optional(
            Output::new(p.PIN_8, Level::Low),        // A
            Output::new(p.PIN_9, Level::Low),        // B
            Output::new(p.PIN_10, Level::Low),       // C
            Some(Output::new(p.PIN_11, Level::Low)), // D
            None,                                    // E not needed for 64x32
        )
        .control(
            Output::new(p.PIN_12, Level::Low),  // CLK
            Output::new(p.PIN_13, Level::Low),  // LAT
            Output::new(p.PIN_14, Level::High), // OE
        )
        .build()
}

/// Common display configuration
pub fn configure_display<P>(
    mut display: hub75_embassy::Hub75Display<P, 64, 32, 6>,
) -> hub75_embassy::Hub75Display<P, 64, 32, 6>
where
    P: embedded_hal::digital::OutputPin,
{
    display.set_double_buffering(true);
    display.set_brightness(hub75_embassy::display::Brightness::new(128)); // 50% brightness
    display.set_refresh_interval(embassy_time::Duration::from_micros(50)); // 20kHz base rate
    display
}
