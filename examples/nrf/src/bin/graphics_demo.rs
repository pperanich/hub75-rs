//! Graphics demo example for nRF52 series microcontrollers
//!
//! This example demonstrates:
//! - Advanced embedded-graphics features
//! - Complex shapes and patterns
//! - Performance optimization techniques
//!
//! Hardware connections: Same as basic_display.rs

#![no_std]
#![no_main]


use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_5X8, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{
        Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, Triangle,
    },
    text::Text,
};
use hub75::{Hub75Display, Hub75Pins, Hub75RgbPins, Hub75AddressPins, Hub75ControlPins};
use {defmt_rtt as _, panic_probe as _};

type Display = Hub75Display<Output<'static>, 32, 32, 4>;

// #[embassy_executor::task]
async fn graphics_demo_task(mut display: Display) {
    info!("Starting graphics demo task");
    
    display.set_double_buffering(true);
    
    let mut demo_phase = 0u32;
    let mut frame = 0u32;
    
    loop {
        display.clear();
        
        defmt::info!("HELLO!");
        match demo_phase {
            0 => geometric_shapes_demo(&mut display, frame),
            1 => line_patterns_demo(&mut display, frame),
            2 => concentric_circles_demo(&mut display, frame),
            3 => triangle_wave_demo(&mut display, frame),
            _ => {
                demo_phase = 0;
                continue;
            }
        }
        
        // Show demo info
        let mut info_text = heapless::String::<16>::new();
        core::fmt::write(&mut info_text, format_args!("Demo {}", demo_phase + 1)).ok();
        Text::new(
            &info_text,
            Point::new(2, 30),
            MonoTextStyle::new(&FONT_5X8, Rgb565::WHITE),
        )
        .draw(&mut display)
        .ok();
        
        display.swap_buffers();
        
        frame += 1;
        
        // Switch demo every 5 seconds (100 frames at 50ms)
        if frame >= 100 {
            frame = 0;
            demo_phase += 1;
        }
        
        Timer::after(Duration::from_millis(50)).await;
    }
}

fn geometric_shapes_demo(display: &mut Display, frame: u32) {
    let offset = (frame / 2) % 20;
    
    // Moving rectangles
    Rectangle::new(Point::new(offset as i32, 2), Size::new(8, 6))
        .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::RED).build())
        .draw(display)
        .ok();
    
    Rectangle::new(Point::new((40 - offset) as i32, 8), Size::new(8, 6))
        .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::GREEN).build())
        .draw(display)
        .ok();
    
    // Pulsing circle
    let radius = 3; // + (frame / 5) % 5;
    Circle::new(Point::new(52, 10), radius * 2)
        .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::BLUE).build())
        .draw(display)
        .ok();
    
    // Static triangle
    Triangle::new(Point::new(10, 20), Point::new(20, 20), Point::new(15, 15))
        .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::YELLOW).build())
        .draw(display)
        .ok();
}

fn line_patterns_demo(display: &mut Display, frame: u32) {
    let colors = [Rgb565::RED, Rgb565::GREEN, Rgb565::BLUE, Rgb565::CYAN, Rgb565::MAGENTA];
    
    // Radiating lines from center
    let center_x = 32;
    let center_y = 12;
    
    for i in 0..8 {
        let angle = (frame + i * 12) % 360;
        let angle_rad = angle as f32 * 3.14159 / 180.0;
        
        let end_x = center_x + (20.0 * angle_rad.cos()) as i32;
        let end_y = center_y + (10.0 * angle_rad.sin()) as i32;
        
        Line::new(Point::new(center_x, center_y), Point::new(end_x, end_y))
            .into_styled(PrimitiveStyle::with_stroke(colors[i as usize % colors.len()], 1))
            .draw(display)
            .ok();
    }
}

fn concentric_circles_demo(display: &mut Display, frame: u32) {
    let center = Point::new(32, 12);
    let colors = [Rgb565::RED, Rgb565::YELLOW, Rgb565::GREEN, Rgb565::CYAN, Rgb565::BLUE];
    
    for i in 0..5 {
        let radius = 2 + i * 3 + (frame / 3) % 6;
        let color = colors[i as usize];
        
        Circle::new(Point::new(center.x - radius as i32, center.y - radius as i32), radius * 2)
            .into_styled(PrimitiveStyle::with_stroke(color, 1))
            .draw(display)
            .ok();
    }
}

fn triangle_wave_demo(display: &mut Display, frame: u32) {
    // Draw a sine wave using triangles
    for x in 0..64 {
        let wave_phase = (x + frame) as f32 * 0.2;
        let y = 12.0 + 8.0 * (wave_phase * 3.14159 / 20.0).sin();
        
        let color = match x % 3 {
            0 => Rgb565::RED,
            1 => Rgb565::GREEN,
            _ => Rgb565::BLUE,
        };
        
        Triangle::new(
            Point::new(x as i32, y as i32),
            Point::new(x as i32 + 2, y as i32 + 3),
            Point::new(x as i32 - 2, y as i32 + 3),
        )
        .into_styled(PrimitiveStyleBuilder::new().fill_color(color).build())
        .draw(display)
        .ok();
    }
}

// Simple trigonometric functions for embedded use
trait FloatExt {
    fn sin(self) -> f32;
    fn cos(self) -> f32;
}

impl FloatExt for f32 {
    fn sin(self) -> f32 {
        let x = self % (2.0 * 3.14159);
        let x2 = x * x;
        x - (x2 * x) / 6.0 + (x2 * x2 * x) / 120.0
    }
    
    fn cos(self) -> f32 {
        (self + 3.14159 / 2.0).sin()
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("nRF HUB75 Graphics Demo Example");

    // Configure HUB75 pins using new nested structure
    let pins = Hub75Pins {
        rgb: Hub75RgbPins {
            r1: Output::new(p.P0_07, Level::Low, OutputDrive::Standard),
            g1: Output::new(p.P0_03, Level::Low, OutputDrive::Standard),
            b1: Output::new(p.P0_05, Level::Low, OutputDrive::Standard),
            r2: Output::new(p.P0_04, Level::Low, OutputDrive::Standard),
            g2: Output::new(p.P0_02, Level::Low, OutputDrive::Standard),
            b2: Output::new(p.P0_06, Level::Low, OutputDrive::Standard),
        },
        address: Hub75AddressPins {
            a: Output::new(p.P0_27, Level::Low, OutputDrive::Standard),
            b: Output::new(p.P1_08, Level::Low, OutputDrive::Standard),
            c: Output::new(p.P1_09, Level::Low, OutputDrive::Standard),
            d: Some(Output::new(p.P0_26, Level::Low, OutputDrive::Standard)),
            e: None,
        },
        control: Hub75ControlPins {
            clk: Output::new(p.P0_08, Level::Low, OutputDrive::Standard),
            lat: Output::new(p.P0_24, Level::Low, OutputDrive::Standard),
            oe: Output::new(p.P0_25, Level::High, OutputDrive::Standard),
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

    // spawner.spawn(graphics_demo_task(display)).unwrap();
    graphics_demo_task(display).await;

    info!("Graphics demo started");
    
    loop {
        Timer::after(Duration::from_secs(5)).await;
        info!("Graphics demo running...");
    }
}