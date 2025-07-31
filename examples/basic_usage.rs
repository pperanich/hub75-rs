//! Basic usage example for the HUB75 Embassy driver
//!
//! This example demonstrates:
//! - Setting up a HUB75 display
//! - Drawing basic shapes with embedded-graphics
//! - Running the display refresh task

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use hub75_embassy::{Hub75Display, Hub75Pins};
use panic_halt as _;

// This example assumes you're using an RP2040, but the pattern works for any embassy-supported MCU
use embassy_rp::{
    gpio::{Level, Output},
    peripherals::*,
};

type Display = Hub75Display<Output<'static>, 64, 32, 6>;

#[embassy_executor::task]
async fn display_refresh_task(mut display: Display) {
    // Continuous refresh loop
    display.refresh_task().await;
}

#[embassy_executor::task]
async fn graphics_task(mut display: Display) {
    // Enable double buffering for smooth updates
    display.set_double_buffering(true);

    let mut counter = 0u32;

    loop {
        // Clear the back buffer
        display.clear();

        // Draw a rectangle
        Rectangle::new(Point::new(2, 2), Size::new(20, 12))
            .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::RED).build())
            .draw(&mut display)
            .unwrap();

        // Draw a circle
        Circle::new(Point::new(35, 5), 10)
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .stroke_color(Rgb565::GREEN)
                    .stroke_width(2)
                    .build(),
            )
            .draw(&mut display)
            .unwrap();

        // Draw counter text
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::BLUE);
        let mut text_buffer = heapless::String::<32>::new();
        core::write!(&mut text_buffer, "Count: {}", counter).unwrap();

        Text::new(&text_buffer, Point::new(2, 25), text_style)
            .draw(&mut display)
            .unwrap();

        // Swap buffers to display the new frame
        display.swap_buffers();

        counter += 1;
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Configure HUB75 pins for a 64x32 display
    let pins = Hub75Pins::new_64x32(
        // RGB pins for upper half (R1, G1, B1)
        Output::new(p.PIN_2, Level::Low),
        Output::new(p.PIN_3, Level::Low),
        Output::new(p.PIN_4, Level::Low),
        // RGB pins for lower half (R2, G2, B2)
        Output::new(p.PIN_5, Level::Low),
        Output::new(p.PIN_6, Level::Low),
        Output::new(p.PIN_7, Level::Low),
        // Address pins (A, B, C, D)
        Output::new(p.PIN_8, Level::Low),
        Output::new(p.PIN_9, Level::Low),
        Output::new(p.PIN_10, Level::Low),
        Output::new(p.PIN_11, Level::Low),
        // Control pins (CLK, LAT, OE)
        Output::new(p.PIN_12, Level::Low),
        Output::new(p.PIN_13, Level::Low),
        Output::new(p.PIN_14, Level::High), // OE is active low
    );

    // Create the display
    let display = Hub75Display::<_, 64, 32, 6>::new(pins).unwrap();

    // Clone display for both tasks (in a real implementation, you'd use embassy-sync)
    // For this example, we'll just run the graphics task
    spawner.spawn(graphics_task(display)).unwrap();

    // Keep the main task alive
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
