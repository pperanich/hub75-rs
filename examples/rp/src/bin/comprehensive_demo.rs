//! Comprehensive RP2040 HUB75 display example
//!
//! This example demonstrates:
//! - Advanced graphics and animations
//! - Dynamic brightness control
//! - Multiple visual effects
//! - Performance monitoring

#![no_std]
#![no_main]


use defmt::*;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use hub75::{display::Brightness, Hub75Display, Hub75Pins, Hub75RgbPins, Hub75AddressPins, Hub75ControlPins};
use {defmt_rtt as _, panic_halt as _};

use embassy_rp::gpio::{Level, Output};

type Display = Hub75Display<Output<'static>, 64, 32, 6>;

#[embassy_executor::task]
async fn comprehensive_demo_task(mut display: Display) {
    info!("Starting comprehensive demo task");
    
    display.set_double_buffering(true);
    
    let mut counter = 0u32;
    let mut brightness_dir = 1i8;
    let mut current_brightness = 128u8;
    let mut demo_mode = 0u8;
    let mut mode_timer = 0u32;

    loop {
        // Update brightness
        if brightness_dir > 0 {
            current_brightness = current_brightness.saturating_add(2);
            if current_brightness >= 250 {
                brightness_dir = -1;
            }
        } else {
            current_brightness = current_brightness.saturating_sub(2);
            if current_brightness <= 50 {
                brightness_dir = 1;
            }
        }
        display.set_brightness(Brightness::new(current_brightness));

        // Clear the display
        display.clear();

        // Run different demo modes
        match demo_mode {
            0 => basic_shapes_demo(&mut display, counter),
            1 => animated_patterns_demo(&mut display, counter),
            2 => text_effects_demo(&mut display, counter, current_brightness),
            _ => {
                demo_mode = 0;
                continue;
            }
        }

        // Draw status info
        let mut text_buffer = heapless::String::<32>::new();
        core::fmt::write(&mut text_buffer, format_args!("M:{} B:{}", demo_mode, current_brightness)).unwrap();
        
        Text::new(
            &text_buffer,
            Point::new(2, 30),
            MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE),
        )
        .draw(&mut display)
        .unwrap();

        // Swap buffers
        display.swap_buffers();

        counter += 1;
        mode_timer += 1;
        
        // Switch demo modes every 5 seconds (100 frames at 50ms)
        if mode_timer >= 100 {
            mode_timer = 0;
            demo_mode = (demo_mode + 1) % 3;
            info!("Switching to demo mode {}", demo_mode);
        }

        Timer::after(Duration::from_millis(50)).await;
    }
}

fn basic_shapes_demo(display: &mut Display, frame: u32) {
    // Animated rectangle
    let rect_x = ((frame / 2) % 48) as i32;
    Rectangle::new(Point::new(rect_x, 2), Size::new(16, 12))
        .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::RED).build())
        .draw(display)
        .unwrap();

    // Pulsing circle
    let radius = 5 + ((frame / 4) % 8) as u32;
    Circle::new(
        Point::new(32 - radius as i32 / 2, 16 - radius as i32 / 2),
        radius,
    )
    .into_styled(
        PrimitiveStyleBuilder::new()
            .stroke_color(Rgb565::GREEN)
            .stroke_width(1)
            .build(),
    )
    .draw(display)
    .unwrap();
}

fn animated_patterns_demo(display: &mut Display, frame: u32) {
    // Moving diagonal lines
    for i in 0..8 {
        let x = ((frame + i * 8) % 80) as i32 - 8;
        Rectangle::new(Point::new(x, i as i32 * 3), Size::new(2, 24))
            .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::CYAN).build())
            .draw(display)
            .ok();
    }
    
    // Bouncing squares (simplified without sin)
    for i in 0..3 {
        let bounce_cycle = ((frame + i * 20) / 10) % 16;
        let y = if bounce_cycle < 8 { bounce_cycle } else { 16 - bounce_cycle };
        Rectangle::new(Point::new(10 + i as i32 * 18, 8 + y as i32), Size::new(6, 6))
            .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::MAGENTA).build())
            .draw(display)
            .ok();
    }
}

fn text_effects_demo(display: &mut Display, frame: u32, brightness: u8) {
    // Scrolling text
    let scroll_x = 64 - ((frame / 2) % 120) as i32;
    Text::new(
        "COMPREHENSIVE DEMO",
        Point::new(scroll_x, 10),
        MonoTextStyle::new(&FONT_6X10, Rgb565::YELLOW),
    )
    .draw(display)
    .ok();
    
    // Blinking brightness indicator
    if (frame / 10) % 2 == 0 {
        let mut brightness_text = heapless::String::<16>::new();
        core::fmt::write(&mut brightness_text, format_args!("BR:{}", brightness)).unwrap();
        Text::new(
            &brightness_text,
            Point::new(20, 20),
            MonoTextStyle::new(&FONT_6X10, Rgb565::GREEN),
        )
        .draw(display)
        .ok();
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Configure HUB75 pins for a 64x32 display
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
            oe: Output::new(p.PIN_14, Level::High),
        },
    };

    // Create the display
    let display = match Hub75Display::<_, 64, 32, 6>::new(pins) {
        Ok(display) => display,
        Err(e) => {
            error!("Failed to create display: {:?}", e);
            return;
        }
    };
    info!("Display initialized");

    // Spawn the comprehensive demo task
    spawner.spawn(comprehensive_demo_task(display)).unwrap();

    info!("Comprehensive demo started");
    
    // Main task loop
    loop {
        Timer::after(Duration::from_secs(5)).await;
        info!("Comprehensive demo running...");
    }
}

// Note: Using panic-halt crate for panic handling
