//! Performance testing example for RP2040/RP2350
//!
//! This example demonstrates:
//! - Performance measurement and optimization
//! - Different refresh rates and color depths
//! - Memory usage monitoring
//! - Frame rate analysis
//!
//! Hardware connections: Same as basic_display.rs

#![no_std]
#![no_main]


use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_5X8, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use hub75_embassy::{Hub75Display, Hub75Pins, Hub75RgbPins, Hub75AddressPins, Hub75ControlPins};
use {defmt_rtt as _, panic_halt as _};

type Display = Hub75Display<Output<'static>, 64, 32, 6>;

struct PerformanceStats {
    frame_count: u32,
    last_fps_update: Instant,
    current_fps: u32,
    min_frame_time: u64,
    max_frame_time: u64,
    avg_frame_time: u64,
}

impl PerformanceStats {
    fn new() -> Self {
        Self {
            frame_count: 0,
            last_fps_update: Instant::now(),
            current_fps: 0,
            min_frame_time: u64::MAX,
            max_frame_time: 0,
            avg_frame_time: 0,
        }
    }

    fn update(&mut self, frame_time_us: u64) {
        self.frame_count += 1;
        
        // Update frame time statistics
        if frame_time_us < self.min_frame_time {
            self.min_frame_time = frame_time_us;
        }
        if frame_time_us > self.max_frame_time {
            self.max_frame_time = frame_time_us;
        }
        
        // Simple moving average
        self.avg_frame_time = (self.avg_frame_time * 7 + frame_time_us) / 8;
        
        // Update FPS every second
        let now = Instant::now();
        if now.duration_since(self.last_fps_update) >= Duration::from_secs(1) {
            self.current_fps = self.frame_count;
            self.frame_count = 0;
            self.last_fps_update = now;
            
            info!("FPS: {}, Avg frame time: {}μs, Min: {}μs, Max: {}μs", 
                  self.current_fps, self.avg_frame_time, self.min_frame_time, self.max_frame_time);
        }
    }
}

#[embassy_executor::task]
async fn performance_test_task(mut display: Display) {
    info!("Starting performance test task");
    
    display.set_double_buffering(true);
    
    let mut stats = PerformanceStats::new();
    let mut test_mode = 0u8;
    let mut mode_timer = 0u32;
    
    loop {
        let frame_start = Instant::now();
        
        display.clear();
        
        match test_mode {
            0 => solid_color_test(&mut display, mode_timer),
            1 => gradient_test(&mut display, mode_timer),
            2 => checkerboard_test(&mut display, mode_timer),
            3 => random_pixels_test(&mut display, mode_timer),
            4 => moving_rectangles_test(&mut display, mode_timer),
            _ => {
                test_mode = 0;
                continue;
            }
        }
        
        // Display performance stats
        display_stats(&mut display, &stats, test_mode);
        
        display.swap_buffers();
        
        let frame_time = frame_start.elapsed();
        stats.update(frame_time.as_micros());
        
        mode_timer += 1;
        
        // Switch test modes every 5 seconds (100 frames at 50ms)
        if mode_timer >= 100 {
            mode_timer = 0;
            test_mode = (test_mode + 1) % 5;
            info!("Switching to test mode {}", test_mode);
            
            // Reset min/max stats for new test
            stats.min_frame_time = u64::MAX;
            stats.max_frame_time = 0;
        }
        
        Timer::after(Duration::from_millis(50)).await;
    }
}

fn solid_color_test(display: &mut Display, frame: u32) {
    let colors = [Rgb565::RED, Rgb565::GREEN, Rgb565::BLUE, Rgb565::WHITE];
    let color = colors[((frame / 25) % colors.len() as u32) as usize];
    
    Rectangle::new(Point::new(0, 0), Size::new(64, 24))
        .into_styled(PrimitiveStyleBuilder::new().fill_color(color).build())
        .draw(display)
        .ok();
}

fn gradient_test(display: &mut Display, frame: u32) {
    for x in 0..64 {
        let intensity = (x * 255 / 63) as u8;
        let color = match (frame / 25) % 3 {
            0 => Rgb565::new(intensity >> 3, 0, 0),
            1 => Rgb565::new(0, intensity >> 2, 0),
            _ => Rgb565::new(0, 0, intensity >> 3),
        };
        
        Rectangle::new(Point::new(x as i32, 0), Size::new(1, 24))
            .into_styled(PrimitiveStyleBuilder::new().fill_color(color).build())
            .draw(display)
            .ok();
    }
}

fn checkerboard_test(display: &mut Display, frame: u32) {
    let offset = (frame / 5) % 8;
    
    for y in 0..24 {
        for x in 0..64 {
            let checker = ((x + y + offset) / 4) % 2 == 0;
            let color = if checker { Rgb565::WHITE } else { Rgb565::BLACK };
            
            Rectangle::new(Point::new(x as i32, y as i32), Size::new(1, 1))
                .into_styled(PrimitiveStyleBuilder::new().fill_color(color).build())
                .draw(display)
                .ok();
        }
    }
}

fn random_pixels_test(display: &mut Display, frame: u32) {
    // Pseudo-random pattern based on frame
    let mut seed = frame.wrapping_mul(1103515245).wrapping_add(12345);
    
    for y in 0..24 {
        for x in 0..64 {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let intensity = (seed >> 16) as u8;
            
            let color = Rgb565::new(
                (intensity & 0x1F) >> 0,
                (intensity & 0x3F) >> 1,
                (intensity & 0x1F) >> 0,
            );
            
            Rectangle::new(Point::new(x as i32, y as i32), Size::new(1, 1))
                .into_styled(PrimitiveStyleBuilder::new().fill_color(color).build())
                .draw(display)
                .ok();
        }
    }
}

fn moving_rectangles_test(display: &mut Display, frame: u32) {
    let colors = [Rgb565::RED, Rgb565::GREEN, Rgb565::BLUE, Rgb565::YELLOW, Rgb565::CYAN];
    
    for i in 0..5 {
        let x = ((frame + i * 20) % 80) as i32 - 8;
        let y = (i * 4) as i32;
        
        Rectangle::new(Point::new(x, y), Size::new(8, 4))
            .into_styled(PrimitiveStyleBuilder::new().fill_color(colors[i as usize]).build())
            .draw(display)
            .ok();
    }
}

fn display_stats(display: &mut Display, stats: &PerformanceStats, test_mode: u8) {
    let test_names = ["Solid", "Grad", "Check", "Rand", "Move"];
    
    // Test mode
    let mut mode_text = heapless::String::<16>::new();
    core::fmt::write(&mut mode_text, format_args!("{}", test_names[test_mode as usize])).ok();
    Text::new(
        &mode_text,
        Point::new(2, 30),
        MonoTextStyle::new(&FONT_5X8, Rgb565::WHITE),
    )
    .draw(display)
    .ok();
    
    // FPS
    let mut fps_text = heapless::String::<16>::new();
    core::fmt::write(&mut fps_text, format_args!("{}fps", stats.current_fps)).ok();
    Text::new(
        &fps_text,
        Point::new(25, 30),
        MonoTextStyle::new(&FONT_5X8, Rgb565::GREEN),
    )
    .draw(display)
    .ok();
    
    // Average frame time
    let mut time_text = heapless::String::<16>::new();
    core::fmt::write(&mut time_text, format_args!("{}us", stats.avg_frame_time)).ok();
    Text::new(
        &time_text,
        Point::new(45, 30),
        MonoTextStyle::new(&FONT_5X8, Rgb565::CYAN),
    )
    .draw(display)
    .ok();
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    info!("RP2040 HUB75 Performance Test Example");

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
    info!("Display initialized for performance testing");

    spawner.spawn(performance_test_task(display)).unwrap();

    info!("Performance test started");
    
    loop {
        Timer::after(Duration::from_secs(10)).await;
        info!("Performance test running...");
    }
}