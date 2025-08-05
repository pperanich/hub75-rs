//! Animated patterns example for nRF52 series microcontrollers
//!
//! This example demonstrates:
//! - Creating animated visual effects
//! - Using random number generation for dynamic patterns
//! - Smooth color transitions and movement
//!
//! Hardware connections: Same as basic_display.rs

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle},
};
use hub75::{Hub75Display, Hub75Pins, Hub75RgbPins, Hub75AddressPins, Hub75ControlPins};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use {defmt_rtt as _, panic_probe as _};

use core::ops::DerefMut;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use static_cell::StaticCell;

type Display = Hub75Display<Output<'static>, 32, 32, 1>;

static DISPLAY: StaticCell<Mutex<NoopRawMutex, Display>> = StaticCell::new();

#[embassy_executor::task]
pub async fn refresh_task(display_handle: &'static Mutex<NoopRawMutex, Display>) -> ! {

    defmt::info!("Starting refresh task");
    let mut delay = embassy_time::Delay;

    loop {
        {
            let mut display = display_handle.lock().await;
            let _ = display.render_frame(&mut delay).await;
        }
        Timer::after(Duration::from_millis(5)).await;
    }
}

#[embassy_executor::task]
async fn animation_task(display_handle: &'static Mutex<NoopRawMutex, Display>) {
    info!("Starting animation task");
    
    let mut rng = ChaCha8Rng::seed_from_u64(12345);
    let mut frame = 0u32;

    loop {
        {
            let mut display_guard = display_handle.lock().await;
            let mut display = display_guard.deref_mut();
            display.clear();

            // Pattern 1: Moving rainbow bars
            if frame < 300 {
                rainbow_bars(&mut display, frame);
            }
            // Pattern 2: Bouncing balls
            else if frame < 600 {
                bouncing_balls(&mut display, frame - 300, &mut rng);
            }
            // Pattern 3: Plasma effect
            else if frame < 900 {
                plasma_effect(&mut display, frame - 600);
            }
            // Pattern 4: Random sparkles
            else {
                random_sparkles(&mut display, &mut rng);
                if frame > 1200 {
                    frame = 0; // Reset cycle
                }
            }

            display.swap_buffers();
        }
        frame += 1;
        Timer::after(Duration::from_millis(50)).await;
    }
}

fn rainbow_bars(display: &mut Display, frame: u32) {
    let offset = (frame / 2) % 32;
    
    for x in 0..32 {
        let hue = ((x + offset) * 6) % 360;
        let color = hsv_to_rgb565(hue as u16, 255, 255);
        
        Rectangle::new(Point::new(x as i32, 0), Size::new(1, 32))
            .into_styled(PrimitiveStyleBuilder::new().fill_color(color).build())
            .draw(display)
            .ok();
    }
}

fn bouncing_balls(display: &mut Display, frame: u32, _rng: &mut ChaCha8Rng) {
    // Static ball positions and velocities (simplified)
    let ball_count = 3;
    let colors = [Rgb565::RED, Rgb565::GREEN, Rgb565::BLUE];
    
    for i in 0..ball_count {
        let phase = frame as f32 * 0.1 + i as f32 * 2.0;
        let x = (8.0 + 10.0 * (phase * 0.7).sin()) as i32;
        let y = (8.0 + 10.0 * (phase).sin()) as i32;
        
        Circle::new(Point::new(x - 3, y - 3), 6)
            .into_styled(PrimitiveStyleBuilder::new().fill_color(colors[i]).build())
            .draw(display)
            .ok();
    }
}

fn plasma_effect(display: &mut Display, frame: u32) {
    let time = frame as f32 * 0.1;
    
    for y in 0..16 {
        for x in 0..16 {
            let fx = x as f32;
            let fy = y as f32;
            
            // Plasma calculation (simplified integer math)
            let v1 = (fx * 0.1 + time).sin();
            let v2 = ((fx + fy) * 0.08 + time * 1.2).sin();
            let v3 = ((fx - fy) * 0.12 + time * 0.8).sin();
            let plasma = (v1 + v2 + v3) * 127.0 + 128.0;
            
            let hue = (plasma as u16) % 360;
            let color = hsv_to_rgb565(hue, 255, 200);
            
            Rectangle::new(Point::new(x*2 as i32, y*2 as i32), Size::new(2, 2))
                .into_styled(PrimitiveStyleBuilder::new().fill_color(color).build())
                .draw(display)
                .ok();
        }
    }
}

fn random_sparkles(display: &mut Display, rng: &mut ChaCha8Rng) {
    // Generate random sparkles
    for _ in 0..20 {
        let x = (rng.next_u32() % 32) as i32;
        let y = (rng.next_u32() % 32) as i32;
        let brightness = (rng.next_u32() % 256) as u8;
        
        let color = Rgb565::new(brightness >> 3, brightness >> 2, brightness >> 3);
        
        Rectangle::new(Point::new(x, y), Size::new(1, 1))
            .into_styled(PrimitiveStyleBuilder::new().fill_color(color).build())
            .draw(display)
            .ok();
    }
}

// Simple HSV to RGB565 conversion
fn hsv_to_rgb565(h: u16, s: u8, v: u8) -> Rgb565 {
    let h = h % 360;
    let s = s as u16;
    let v = v as u16;
    
    let c = (v * s) / 255;
    let x = c * (60 - ((h % 120) as i16 - 60).abs() as u16) / 60;
    let m = v - c;
    
    let (r, g, b) = match h / 60 {
        0 => (c, x, 0),
        1 => (x, c, 0),
        2 => (0, c, x),
        3 => (0, x, c),
        4 => (x, 0, c),
        _ => (c, 0, x),
    };
    
    let r = ((r + m) >> 3) as u8;
    let g = ((g + m) >> 2) as u8;
    let b = ((b + m) >> 3) as u8;
    
    Rgb565::new(r, g, b)
}

// Simplified sin function using lookup table
trait FloatExt {
    fn sin(self) -> f32;
}

impl FloatExt for f32 {
    fn sin(self) -> f32 {
        // Very simple sin approximation for embedded use
        let x = self % (2.0 * 3.14159);
        let x2 = x * x;
        x - (x2 * x) / 6.0 + (x2 * x2 * x) / 120.0
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    info!("nRF HUB75 Animated Patterns Example");

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

    let mut display = match Hub75Display::new(pins) {
        Ok(display) => display,
        Err(e) => {
            error!("Failed to create display: {:?}", e);
            return;
        }
    };
    info!("Display initialized");

    // Enable double buffering for smooth updates
    display.set_double_buffering(true);

    let display = DISPLAY.init(Mutex::new(display));

    spawner.spawn(animation_task(display)).unwrap();
    spawner.spawn(refresh_task(display)).unwrap();

    info!("Animation started");
    
    loop {
        Timer::after(Duration::from_secs(5)).await;
        info!("Animation running...");
    }
}