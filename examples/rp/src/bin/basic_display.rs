//! Basic HUB75 display example for RP2040/RP2350
//!
//! This example demonstrates:
//! - Simple HUB75 setup with embassy-rp
//! - Basic shapes and text display
//! - Double buffering for smooth updates
//!
//! Hardware connections (for Raspberry Pi Pico):
//! - R1, G1, B1: GP2, GP3, GP4
//! - R2, G2, B2: GP5, GP6, GP7
//! - A, B, C, D: GP8, GP9, GP10, GP11
//! - CLK: GP12, LAT: GP13, OE: GP14

#![no_std]
#![no_main]


use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use hub75::{Hub75Display, Hub75Pins, Hub75RgbPins, Hub75AddressPins, Hub75ControlPins};
use {defmt_rtt as _, panic_halt as _};

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

        // Show RP2040 info
        Text::new(
            "RP2040",
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
    let p = embassy_rp::init(Default::default());
    info!("RP2040 HUB75 Basic Display Example");

    // Configure HUB75 pins for RP2040
    let pins = Hub75Pins {
        rgb: Hub75RgbPins {
            r1: Output::new(p.PIN_2, Level::Low),
            g1: Output::new(p.PIN_3, Level::Low),
            b1: Output::new(p.PIN_4, Level::Low),
            r2: Output::new(p.PIN_5, Level::Low),
            g2: Output::new(p.PIN_6, Level::Low),
            b2: Output::new(p.PIN_7, Level::Low),
        },
        address: Hub75AddressPins {
            a: Output::new(p.PIN_8, Level::Low),
            b: Output::new(p.PIN_9, Level::Low),
            c: Output::new(p.PIN_10, Level::Low),
            d: Some(Output::new(p.PIN_11, Level::Low)),
            e: None,
        },
        control: Hub75ControlPins {
            clk: Output::new(p.PIN_12, Level::Low),
            lat: Output::new(p.PIN_13, Level::Low),
            oe: Output::new(p.PIN_14, Level::High), // Active low
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

    // Spawn the combined task
    spawner.spawn(combined_display_task(display)).unwrap();

    info!("Tasks spawned, entering main loop");
    
    // Main task can do other work or just sleep
    loop {
        Timer::after(Duration::from_secs(1)).await;
        info!("Main loop tick - system running");
    }
}