//! Text display example for nRF52 series microcontrollers
//!
//! This example demonstrates:
//! - Displaying scrolling text
//! - Using different fonts and colors
//! - Text animation effects
//!
//! Hardware connections: Same as basic_display.rs

#![no_std]
#![no_main]


use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, ascii::FONT_5X8, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::Text,
};
use hub75::{Hub75Display, Hub75Pins, Hub75RgbPins, Hub75AddressPins, Hub75ControlPins};
use {defmt_rtt as _, panic_probe as _};

type Display = Hub75Display<Output<'static>, 64, 32, 6>;

#[embassy_executor::task]
async fn text_animation_task(mut display: Display) {
    info!("Starting text animation task");
    
    display.set_double_buffering(true);
    
    let messages = [
        "Hello nRF!",
        "HUB75 Display",
        "Embassy Async",
        "Rust Embedded",
    ];
    
    let mut message_index = 0;
    let mut scroll_offset = 64i32; // Start off-screen to the right
    
    loop {
        display.clear();
        
        let current_message = messages[message_index];
        
        // Scrolling text effect
        Text::new(
            current_message,
            Point::new(scroll_offset, 10),
            MonoTextStyle::new(&FONT_6X10, Rgb565::CYAN),
        )
        .draw(&mut display)
        .ok();
        
        // Static status text
        Text::new(
            "nRF52",
            Point::new(2, 25),
            MonoTextStyle::new(&FONT_5X8, Rgb565::YELLOW),
        )
        .draw(&mut display)
        .ok();
        
        // Time counter
        let time_ms = embassy_time::Instant::now().as_millis();
        let seconds = (time_ms / 1000) % 60;
        let mut time_str = heapless::String::<8>::new();
        core::fmt::write(&mut time_str, format_args!("{}s", seconds)).ok();
        
        Text::new(
            &time_str,
            Point::new(45, 25),
            MonoTextStyle::new(&FONT_5X8, Rgb565::GREEN),
        )
        .draw(&mut display)
        .ok();
        
        display.swap_buffers();
        
        // Update scroll position
        scroll_offset -= 1;
        
        // Calculate text width (approximate)
        let text_width = current_message.len() as i32 * 6;
        
        // Reset when text has scrolled completely off screen
        if scroll_offset < -text_width {
            scroll_offset = 64;
            message_index = (message_index + 1) % messages.len();
        }
        
        Timer::after(Duration::from_millis(80)).await;
    }
}



#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("nRF HUB75 Text Display Example");

    // Configure HUB75 pins using new nested structure
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
            oe: Output::new(p.P0_14, Level::High, OutputDrive::Standard),
        },
    };

    let display = match Hub75Display::new(pins) {
        Ok(display) => display,
        Err(e) => {
            error!("Failed to create display: {:?}", e);
            return;
        }
    };
    info!("Display initialized");

    spawner.spawn(text_animation_task(display)).unwrap();

    info!("Text animation started");
    
    loop {
        Timer::after(Duration::from_secs(10)).await;
        info!("Text display running...");
    }
}