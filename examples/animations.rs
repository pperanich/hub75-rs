//! Animation example for the HUB75 Embassy driver
//!
//! This example demonstrates:
//! - Creating frame-based animations
//! - Using different animation effects (slide, fade, wipe)
//! - Text scrolling animations

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle},
};
use hub75_embassy::{
    animation::{Animation, AnimationData, AnimationEffect, AnimationState},
    Hub75Color, Hub75Display, Hub75FrameBuffer, Hub75Pins,
};
use panic_halt as _;

// This example assumes you're using an RP2040
use embassy_rp::{
    gpio::{Level, Output},
    peripherals::*,
};

type Display = Hub75Display<Output<'static>, 64, 32, 6>;
type FrameBuffer = Hub75FrameBuffer<64, 32, 6>;

#[embassy_executor::task]
async fn display_refresh_task(mut display: Display) {
    display.refresh_task().await;
}

#[embassy_executor::task]
async fn animation_demo_task(mut display: Display) {
    display.set_double_buffering(true);

    loop {
        // Demo 1: Sliding colored rectangles
        sliding_rectangles_demo(&mut display).await;
        Timer::after(Duration::from_secs(2)).await;

        // Demo 2: Fading circles
        fading_circles_demo(&mut display).await;
        Timer::after(Duration::from_secs(2)).await;

        // Demo 3: Text scrolling
        text_scroll_demo(&mut display).await;
        Timer::after(Duration::from_secs(2)).await;

        // Demo 4: Wipe effect with patterns
        wipe_patterns_demo(&mut display).await;
        Timer::after(Duration::from_secs(2)).await;
    }
}

async fn sliding_rectangles_demo(display: &mut Display) {
    // Create frames with different colored rectangles
    let mut frames = heapless::Vec::<FrameBuffer, 4>::new();

    let colors = [
        Hub75Color::red(),
        Hub75Color::green(),
        Hub75Color::blue(),
        Hub75Color::white(),
    ];

    for (i, &color) in colors.iter().enumerate() {
        let mut frame = FrameBuffer::new();

        // Create a rectangle that moves across the screen
        let x = (i * 16) as usize;
        for dy in 0..16 {
            for dx in 0..16 {
                if x + dx < 64 && dy < 32 {
                    frame.set_pixel(x + dx, dy, color).ok();
                }
            }
        }

        frames.push(frame).ok();
    }

    // Create sliding animation
    let mut animation = Animation::new(
        AnimationData::Frames(&frames),
        AnimationEffect::Slide,
        Duration::from_secs(3),
    )
    .unwrap();

    // Run the animation
    loop {
        match animation.next(embassy_time::Instant::now()) {
            AnimationState::Apply(frame) => {
                display.back_buffer().copy_from(&frame);
                display.swap_buffers();
            }
            AnimationState::Wait => {
                Timer::after(Duration::from_millis(10)).await;
            }
            AnimationState::Done => break,
        }
    }
}

async fn fading_circles_demo(display: &mut Display) {
    // Create frames with circles of different sizes
    let mut frames = heapless::Vec::<FrameBuffer, 8>::new();

    for radius in 1..=8 {
        let mut frame = FrameBuffer::new();

        // Draw a circle in the center
        let center_x = 32;
        let center_y = 16;

        for y in 0..32 {
            for x in 0..64 {
                let dx = (x as i32 - center_x as i32).abs() as u32;
                let dy = (y as i32 - center_y as i32).abs() as u32;
                let distance_sq = dx * dx + dy * dy;

                if distance_sq <= (radius * radius) {
                    let color =
                        Hub75Color::new((radius * 8) as u8, (16 - radius) as u8, radius as u8);
                    frame.set_pixel(x, y, color).ok();
                }
            }
        }

        frames.push(frame).ok();
    }

    // Create fade animation
    let mut animation = Animation::new(
        AnimationData::Frames(&frames),
        AnimationEffect::Fade,
        Duration::from_secs(4),
    )
    .unwrap();

    // Run the animation
    loop {
        match animation.next(embassy_time::Instant::now()) {
            AnimationState::Apply(frame) => {
                display.back_buffer().copy_from(&frame);
                display.swap_buffers();
            }
            AnimationState::Wait => {
                Timer::after(Duration::from_millis(10)).await;
            }
            AnimationState::Done => break,
        }
    }
}

async fn text_scroll_demo(display: &mut Display) {
    // Create text animation
    let mut animation = Animation::new(
        AnimationData::Text("HELLO WORLD"),
        AnimationEffect::Slide,
        Duration::from_secs(5),
    )
    .unwrap();

    // Run the animation
    loop {
        match animation.next(embassy_time::Instant::now()) {
            AnimationState::Apply(frame) => {
                display.back_buffer().copy_from(&frame);
                display.swap_buffers();
            }
            AnimationState::Wait => {
                Timer::after(Duration::from_millis(10)).await;
            }
            AnimationState::Done => break,
        }
    }
}

async fn wipe_patterns_demo(display: &mut Display) {
    // Create frames with different patterns
    let mut frames = heapless::Vec::<FrameBuffer, 3>::new();

    // Pattern 1: Diagonal stripes
    let mut frame1 = FrameBuffer::new();
    for y in 0..32 {
        for x in 0..64 {
            if (x + y) % 8 < 4 {
                frame1.set_pixel(x, y, Hub75Color::red()).ok();
            }
        }
    }
    frames.push(frame1).ok();

    // Pattern 2: Checkerboard
    let mut frame2 = FrameBuffer::new();
    for y in 0..32 {
        for x in 0..64 {
            if (x / 4 + y / 4) % 2 == 0 {
                frame2.set_pixel(x, y, Hub75Color::green()).ok();
            }
        }
    }
    frames.push(frame2).ok();

    // Pattern 3: Concentric rectangles
    let mut frame3 = FrameBuffer::new();
    for y in 0..32 {
        for x in 0..64 {
            let dist_from_edge =
                core::cmp::min(core::cmp::min(x, 63 - x), core::cmp::min(y, 31 - y));
            if dist_from_edge % 4 < 2 {
                frame3.set_pixel(x, y, Hub75Color::blue()).ok();
            }
        }
    }
    frames.push(frame3).ok();

    // Create wipe animation
    let mut animation = Animation::new(
        AnimationData::Frames(&frames),
        AnimationEffect::Wipe,
        Duration::from_secs(4),
    )
    .unwrap();

    // Run the animation
    loop {
        match animation.next(embassy_time::Instant::now()) {
            AnimationState::Apply(frame) => {
                display.back_buffer().copy_from(&frame);
                display.swap_buffers();
            }
            AnimationState::Wait => {
                Timer::after(Duration::from_millis(10)).await;
            }
            AnimationState::Done => break,
        }
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

    // Start the animation demo
    spawner.spawn(animation_demo_task(display)).unwrap();

    // Keep the main task alive
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
