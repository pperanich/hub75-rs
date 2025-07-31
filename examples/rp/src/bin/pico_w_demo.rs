//! Pico W WiFi + HUB75 display example
//!
//! This example demonstrates:
//! - Using WiFi on Pico W alongside HUB75 display
//! - Displaying network information on the LED matrix
//! - Async WiFi operations with display updates
//!
//! Hardware connections: Same as basic_display.rs
//! Note: Requires Pico W board with WiFi capability

#![no_std]
#![no_main]


use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_5X8, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use hub75::{Hub75Display, Hub75Pins, Hub75RgbPins, Hub75AddressPins, Hub75ControlPins};
use {defmt_rtt as _, panic_halt as _};

type Display = Hub75Display<Output<'static>, 64, 32, 6>;

#[derive(Clone, Copy)]
enum WifiStatus {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

#[embassy_executor::task]
async fn wifi_status_task(mut display: Display) {
    info!("Starting WiFi status display task");
    
    display.set_double_buffering(true);
    
    let mut wifi_status = WifiStatus::Disconnected;
    let mut connection_attempts = 0u32;
    let mut uptime_seconds = 0u32;
    let mut animation_frame = 0u32;
    
    loop {
        display.clear();
        
        // Simulate WiFi connection process
        match wifi_status {
            WifiStatus::Disconnected => {
                if animation_frame % 100 == 0 {
                    wifi_status = WifiStatus::Connecting;
                    connection_attempts += 1;
                    info!("Attempting WiFi connection #{}", connection_attempts);
                }
            }
            WifiStatus::Connecting => {
                if animation_frame % 50 == 0 {
                    // Simulate connection success/failure
                    if connection_attempts % 3 == 0 {
                        wifi_status = WifiStatus::Error;
                    } else {
                        wifi_status = WifiStatus::Connected;
                        info!("WiFi connected successfully");
                    }
                }
            }
            WifiStatus::Connected => {
                if animation_frame % 200 == 0 {
                    // Simulate occasional disconnection
                    if (animation_frame / 200) % 10 == 0 {
                        wifi_status = WifiStatus::Disconnected;
                        info!("WiFi disconnected");
                    }
                }
            }
            WifiStatus::Error => {
                if animation_frame % 60 == 0 {
                    wifi_status = WifiStatus::Disconnected;
                    info!("Retrying WiFi connection");
                }
            }
        }
        
        // Draw WiFi status indicator
        draw_wifi_status(&mut display, wifi_status, animation_frame);
        
        // Draw connection info
        draw_connection_info(&mut display, connection_attempts, uptime_seconds);
        
        // Draw animated elements
        draw_signal_strength(&mut display, wifi_status, animation_frame);
        
        display.swap_buffers();
        
        animation_frame += 1;
        
        // Update uptime every second (20 frames at 50ms)
        if animation_frame % 20 == 0 {
            uptime_seconds += 1;
        }
        
        Timer::after(Duration::from_millis(50)).await;
    }
}

fn draw_wifi_status(display: &mut Display, status: WifiStatus, frame: u32) {
    let (text, color) = match status {
        WifiStatus::Disconnected => ("DISC", Rgb565::RED),
        WifiStatus::Connecting => {
            // Blinking effect while connecting
            if (frame / 5) % 2 == 0 {
                ("CONN", Rgb565::YELLOW)
            } else {
                ("....", Rgb565::new(8, 8, 0))
            }
        }
        WifiStatus::Connected => ("WIFI", Rgb565::GREEN),
        WifiStatus::Error => {
            // Fast blinking red for error
            if (frame / 2) % 2 == 0 {
                ("ERR!", Rgb565::RED)
            } else {
                ("    ", Rgb565::BLACK)
            }
        }
    };
    
    Text::new(
        text,
        Point::new(2, 8),
        MonoTextStyle::new(&FONT_5X8, color),
    )
    .draw(display)
    .ok();
}

fn draw_connection_info(display: &mut Display, attempts: u32, uptime: u32) {
    // Connection attempts
    let mut attempts_text = heapless::String::<16>::new();
    core::fmt::write(&mut attempts_text, format_args!("Att:{}", attempts)).ok();
    Text::new(
        &attempts_text,
        Point::new(2, 16),
        MonoTextStyle::new(&FONT_5X8, Rgb565::CYAN),
    )
    .draw(display)
    .ok();
    
    // Uptime
    let minutes = uptime / 60;
    let seconds = uptime % 60;
    let mut uptime_text = heapless::String::<16>::new();
    core::fmt::write(&mut uptime_text, format_args!("{}:{:02}", minutes, seconds)).ok();
    Text::new(
        &uptime_text,
        Point::new(2, 24),
        MonoTextStyle::new(&FONT_5X8, Rgb565::WHITE),
    )
    .draw(display)
    .ok();
    
    // Pico W identifier
    Text::new(
        "PicoW",
        Point::new(35, 24),
        MonoTextStyle::new(&FONT_5X8, Rgb565::MAGENTA),
    )
    .draw(display)
    .ok();
}

fn draw_signal_strength(display: &mut Display, status: WifiStatus, frame: u32) {
    if matches!(status, WifiStatus::Connected) {
        // Animated signal strength bars
        let base_x = 45;
        let base_y = 16;
        
        for i in 0..4 {
            let bar_height = i + 1;
            let should_show = (frame / (10 + i * 5)) % 8 < (6 - i);
            
            if should_show {
                Rectangle::new(
                    Point::new(base_x + i as i32 * 3, base_y - bar_height as i32),
                    Size::new(2, bar_height + 1),
                )
                .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::GREEN).build())
                .draw(display)
                .ok();
            }
        }
    } else if matches!(status, WifiStatus::Connecting) {
        // Scanning animation
        let scan_pos = (frame / 3) % 16;
        
        Rectangle::new(
            Point::new(45 + scan_pos as i32, 10),
            Size::new(2, 1),
        )
        .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::YELLOW).build())
        .draw(display)
        .ok();
    }
}

// Simulated WiFi task (would be real WiFi operations on actual Pico W)
#[embassy_executor::task]
async fn wifi_manager_task() {
    info!("WiFi manager task started (simulated)");
    
    loop {
        // In a real implementation, this would handle:
        // - WiFi scanning
        // - Connection management
        // - Network requests
        // - Status updates
        
        Timer::after(Duration::from_secs(5)).await;
        info!("WiFi manager tick (simulated)");
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("Pico W HUB75 + WiFi Demo Example");

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

    // Note: In a real Pico W implementation, you would also initialize:
    // - WiFi driver
    // - Network stack
    // - TLS/HTTP clients
    // - etc.

    spawner.spawn(wifi_status_task(display)).unwrap();
    spawner.spawn(wifi_manager_task()).unwrap();

    info!("Pico W demo started");
    
    loop {
        Timer::after(Duration::from_secs(10)).await;
        info!("System running - display + WiFi simulation");
    }
}