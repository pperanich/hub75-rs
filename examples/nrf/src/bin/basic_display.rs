//! Basic HUB75 display example for nRF52 series microcontrollers
//!
//! This example demonstrates:
//! - Setting up a HUB75 display with embassy-nrf
//! - Drawing basic shapes and text
//! - Running the display refresh task
//!
//! Hardware connections (example for 64x32 panel):
//! - R1, G1, B1: P0.02, P0.03, P0.04
//! - R2, G2, B2: P0.05, P0.06, P0.07
//! - A, B, C, D: P0.08, P0.28, P0.29, P0.30
//! - CLK: P0.12
//! - LAT: P0.13
//! - OE: P0.14

#![no_std]
#![no_main]


use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use hub75::{Hub75Display, Hub75Pins, Hub75RgbPins, Hub75AddressPins, Hub75ControlPins};
use {defmt_rtt as _, panic_probe as _};

type Display = Hub75Display<Output<'static>, 64, 32, 6>;

#[embassy_executor::task]
async fn combined_display_task(mut display: Display) {
    info!("Starting combined display and graphics task");
    
    // Enable double buffering for smooth updates
    display.set_double_buffering(true);

    let mut counter = 0u32;
    
    // Create a delay provider using embassy-time
    let mut delay = embassy_time::Delay;

    loop {
        // Clear the back buffer
        display.clear();

        // Draw a red rectangle
        Rectangle::new(Point::new(2, 2), Size::new(20, 12))
            .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::RED).build())
            .draw(&mut display)
            .unwrap();

        // Draw a green circle
        Circle::new(Point::new(30, 8), 10)
            .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::GREEN).build())
            .draw(&mut display)
            .unwrap();

        // Draw a blue rectangle
        Rectangle::new(Point::new(45, 2), Size::new(15, 12))
            .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::BLUE).build())
            .draw(&mut display)
            .unwrap();

        // Draw counter text
        let mut text_buffer = heapless::String::<32>::new();
        core::fmt::write(&mut text_buffer, format_args!("Count: {}", counter)).unwrap();
        
        Text::new(
            &text_buffer,
            Point::new(2, 25),
            MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE),
        )
        .draw(&mut display)
        .unwrap();

        // Show nRF52 info
        Text::new(
            "nRF52",
            Point::new(45, 25),
            MonoTextStyle::new(&FONT_6X10, Rgb565::CYAN),
        )
        .draw(&mut display)
        .unwrap();

        // Swap buffers to display the new frame
        display.swap_buffers();

        // Render the frame to the display
        if let Err(e) = display.render_frame(&mut delay).await {
            error!("Failed to render frame: {:?}", e);
        }

        counter = counter.wrapping_add(1);
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("nRF HUB75 Basic Display Example");

    // Configure HUB75 pins for nRF52 using commonly available pins
    let pins = Hub75Pins {
        rgb: Hub75RgbPins {
            r1: Output::new(p.P0_02, Level::Low, OutputDrive::Standard),
            g1: Output::new(p.P0_03, Level::Low, OutputDrive::Standard),
            b1: Output::new(p.P0_04, Level::Low, OutputDrive::Standard),
            r2: Output::new(p.P0_05, Level::Low, OutputDrive::Standard),
            g2: Output::new(p.P0_06, Level::Low, OutputDrive::Standard),
            b2: Output::new(p.P0_07, Level::Low, OutputDrive::Standard),
        },
        address: Hub75AddressPins {
            a: Output::new(p.P0_08, Level::Low, OutputDrive::Standard),
            b: Output::new(p.P0_28, Level::Low, OutputDrive::Standard),
            c: Output::new(p.P0_29, Level::Low, OutputDrive::Standard),
            d: Some(Output::new(p.P0_30, Level::Low, OutputDrive::Standard)),
            e: None,
        },
        control: Hub75ControlPins {
            clk: Output::new(p.P0_12, Level::Low, OutputDrive::Standard),
            lat: Output::new(p.P0_13, Level::Low, OutputDrive::Standard),
            oe: Output::new(p.P0_14, Level::High, OutputDrive::Standard), // Active low
        },
    };

    // Create the display
    let display = match Hub75Display::new(pins) {
        Ok(display) => display,
        Err(e) => {
            error!("Failed to create display: {:?}", e);
            return;
        }
    };
    info!("Display initialized");

    // Since the display can't be cloned, we need to use a different approach
    // For now, let's combine both tasks into one
    spawner.spawn(combined_display_task(display)).unwrap();

    info!("Tasks spawned, entering main loop");
    
    // Main task can do other work or just sleep
    loop {
        Timer::after(Duration::from_secs(1)).await;
        info!("Main loop tick");
    }
}