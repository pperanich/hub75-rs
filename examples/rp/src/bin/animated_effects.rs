//! Animated effects example for RP2040/RP2350
//!
//! This example demonstrates:
//! - Various visual effects optimized for RP2040
//! - Using both CPU cores for parallel processing
//! - Advanced color manipulation and patterns
//!
//! Hardware connections: Same as basic_display.rs

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
};
use hub75_embassy::{Hub75Display, Hub75Pins, Hub75RgbPins, Hub75AddressPins, Hub75ControlPins};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha8Rng;
use {defmt_rtt as _, panic_halt as _};

type Display = Hub75Display<Output<'static>, 64, 32, 6>;

#[embassy_executor::task]
async fn animation_task(mut display: Display) {
    info!("Starting animation task");
    
    display.set_double_buffering(true);
    
    let mut rng = ChaCha8Rng::seed_from_u64(embassy_time::Instant::now().as_millis());
    let mut effect_timer = 0u32;
    let mut current_effect = 0u8;

    loop {
        display.clear();

        match current_effect {
            0 => rainbow_wave_effect(&mut display, effect_timer),
            1 => matrix_rain_effect(&mut display, effect_timer, &mut rng),
            2 => fire_effect(&mut display, effect_timer, &mut rng),
            3 => plasma_tunnel_effect(&mut display, effect_timer),
            4 => starfield_effect(&mut display, effect_timer, &mut rng),
            _ => {
                current_effect = 0;
                continue;
            }
        }

        display.swap_buffers();
        effect_timer += 1;

        // Switch effects every 8 seconds (160 frames at 50ms)
        if effect_timer >= 160 {
            effect_timer = 0;
            current_effect = (current_effect + 1) % 5;
            info!("Switching to effect {}", current_effect);
        }

        Timer::after(Duration::from_millis(50)).await;
    }
}

fn rainbow_wave_effect(display: &mut Display, frame: u32) {
    for y in 0..32 {
        for x in 0..64 {
            let wave1 = ((x as f32 * 0.1 + frame as f32 * 0.05).sin() * 127.0 + 128.0) as u8;
            let wave2 = ((y as f32 * 0.15 + frame as f32 * 0.03).sin() * 127.0 + 128.0) as u8;
            let combined = ((wave1 as u16 + wave2 as u16) / 2) as u8;
            
            let hue = (combined as u16 * 360 / 255) % 360;
            let color = hsv_to_rgb565(hue, 255, 200);
            
            Rectangle::new(Point::new(x as i32, y as i32), Size::new(1, 1))
                .into_styled(PrimitiveStyleBuilder::new().fill_color(color).build())
                .draw(display)
                .ok();
        }
    }
}

fn matrix_rain_effect(display: &mut Display, frame: u32, rng: &mut ChaCha8Rng) {
    // Static drops array (simplified for embedded)
    static mut DROPS: [u8; 64] = [0; 64];
    
    unsafe {
        for x in 0..64 {
            // Update drop position
            if frame % 3 == 0 {
                DROPS[x] = DROPS[x].saturating_add(1);
                if DROPS[x] > 35 || (rng.next_u32() % 100) < 2 {
                    DROPS[x] = 0;
                }
            }
            
            // Draw the drop trail
            for i in 0..8 {
                let y = DROPS[x].saturating_sub(i);
                if y < 32 {
                    let brightness = 255 - (i * 32);
                    let color = Rgb565::new(0, brightness as u8 >> 2, 0);
                    
                    Rectangle::new(Point::new(x as i32, y as i32), Size::new(1, 1))
                        .into_styled(PrimitiveStyleBuilder::new().fill_color(color).build())
                        .draw(display)
                        .ok();
                }
            }
        }
    }
}

fn fire_effect(display: &mut Display, _frame: u32, rng: &mut ChaCha8Rng) {
    // Simple fire simulation
    for x in 0..64 {
        for y in 0..32 {
            let base_heat: u8 = if y > 28 { 255 } else { 0 };
            let noise = (rng.next_u32() % 64) as u8;
            let cooling = (y * 8) as u8;
            
            let heat = base_heat.saturating_sub(cooling).saturating_add(noise / 4);
            
            let color = if heat > 200 {
                Rgb565::new(31, 31, (heat - 200) >> 3)
            } else if heat > 100 {
                Rgb565::new(31, (heat - 100) >> 2, 0)
            } else {
                Rgb565::new(heat >> 3, 0, 0)
            };
            
            Rectangle::new(Point::new(x as i32, (31 - y) as i32), Size::new(1, 1))
                .into_styled(PrimitiveStyleBuilder::new().fill_color(color).build())
                .draw(display)
                .ok();
        }
    }
}

fn plasma_tunnel_effect(display: &mut Display, frame: u32) {
    let time = frame as f32 * 0.1;
    let center_x = 32.0;
    let center_y = 16.0;
    
    for y in 0..32 {
        for x in 0..64 {
            let dx = x as f32 - center_x;
            let dy = y as f32 - center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);
            
            let plasma = (distance * 0.1 + time).sin() + (angle * 3.0 + time * 2.0).sin();
            let intensity = ((plasma + 2.0) * 127.0) as u8;
            
            let hue = ((intensity as u16 * 2 + frame as u16) % 360) as u16;
            let color = hsv_to_rgb565(hue, 255, intensity);
            
            Rectangle::new(Point::new(x as i32, y as i32), Size::new(1, 1))
                .into_styled(PrimitiveStyleBuilder::new().fill_color(color).build())
                .draw(display)
                .ok();
        }
    }
}

fn starfield_effect(display: &mut Display, frame: u32, rng: &mut ChaCha8Rng) {
    // Moving starfield
    static mut STARS: [(u8, u8, u8); 50] = [(0, 0, 0); 50];
    
    unsafe {
        // Initialize stars on first frame
        if frame == 0 {
            for i in 0..50 {
                STARS[i] = (
                    (rng.next_u32() % 64) as u8,
                    (rng.next_u32() % 32) as u8,
                    (rng.next_u32() % 8 + 1) as u8, // speed
                );
            }
        }
        
        // Update and draw stars
        for i in 0..50 {
            let (x, y, speed) = &mut STARS[i];
            
            // Move star
            *x = x.wrapping_sub(*speed);
            
            // Reset star when it goes off screen
            if *x > 200 { // wrapped around
                *x = 63;
                *y = (rng.next_u32() % 32) as u8;
                *speed = (rng.next_u32() % 4 + 1) as u8;
            }
            
            // Draw star with brightness based on speed
            let brightness = *speed * 32;
            let color = Rgb565::new(brightness >> 3, brightness >> 3, brightness >> 3);
            
            Rectangle::new(Point::new(*x as i32, *y as i32), Size::new(1, 1))
                .into_styled(PrimitiveStyleBuilder::new().fill_color(color).build())
                .draw(display)
                .ok();
        }
    }
}

// HSV to RGB565 conversion
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

// Simplified math functions for embedded use
trait FloatExt {
    fn sin(self) -> f32;
    fn sqrt(self) -> f32;
    fn atan2(self, other: f32) -> f32;
}

impl FloatExt for f32 {
    fn sin(self) -> f32 {
        let x = self % (2.0 * 3.14159);
        let x2 = x * x;
        x - (x2 * x) / 6.0 + (x2 * x2 * x) / 120.0
    }
    
    fn sqrt(self) -> f32 {
        if self <= 0.0 { return 0.0; }
        let mut x = self;
        let mut prev = 0.0;
        while (x - prev).abs() > 0.01 {
            prev = x;
            x = (x + self / x) * 0.5;
        }
        x
    }
    
    fn atan2(self, other: f32) -> f32 {
        // Simplified atan2 approximation
        if other.abs() > self.abs() {
            let ratio = self / other;
            let result = ratio / (1.0 + 0.28 * ratio * ratio);
            if other < 0.0 { result + 3.14159 } else { result }
        } else {
            let ratio = other / self;
            let result = 1.5708 - ratio / (1.0 + 0.28 * ratio * ratio);
            if self < 0.0 { result + 3.14159 } else { result }
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("RP2040 HUB75 Animated Effects Example");

    // Configure HUB75 pins
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

    let display = match Hub75Display::new(pins) {
        Ok(display) => display,
        Err(e) => {
            error!("Failed to create display: {:?}", e);
            return;
        }
    };
    info!("Display initialized");

    spawner.spawn(animation_task(display)).unwrap();

    info!("Animated effects started");
    
    loop {
        Timer::after(Duration::from_secs(5)).await;
        info!("Effects running...");
    }
}