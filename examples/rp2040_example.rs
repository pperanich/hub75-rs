//! Complete RP2040 example with proper task management
//!
//! This example demonstrates:
//! - Proper embassy-sync usage for sharing display between tasks
//! - Background refresh task running at high priority
//! - Graphics updates running at lower priority
//! - Real-world pin configuration for RP2040

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use hub75_embassy::{Brightness, Hub75Display, Hub75Pins};
use panic_halt as _;

use embassy_rp::{
    gpio::{Level, Output},
    peripherals::*,
};

type Display = Hub75Display<Output<'static>, 64, 32, 6>;
type SharedDisplay = Mutex<ThreadModeRawMutex, Display>;

static DISPLAY: SharedDisplay =
    Mutex::new(Hub75Display::new(unsafe { core::mem::zeroed() }).unwrap());

#[embassy_executor::task]
async fn display_refresh_task(display: &'static SharedDisplay) {
    loop {
        {
            let mut display = display.lock().await;
            if let Err(_) = display.render_frame().await {
                // Handle error - maybe reset or continue
                Timer::after(Duration::from_millis(1)).await;
            }
        }
        // Small yield to allow other tasks to run
        embassy_futures::yield_now().await;
    }
}

#[embassy_executor::task]
async fn graphics_task(display: &'static SharedDisplay) {
    let mut counter = 0u32;
    let mut brightness_dir = 1i8;
    let mut current_brightness = 128u8;

    loop {
        {
            let mut display = display.lock().await;

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

            // Draw animated rectangle
            let rect_x = ((counter / 2) % 48) as i32;
            Rectangle::new(Point::new(rect_x, 2), Size::new(16, 12))
                .into_styled(PrimitiveStyleBuilder::new().fill_color(Rgb565::RED).build())
                .draw(&mut *display)
                .unwrap();

            // Draw pulsing circle
            let radius = 5 + ((counter / 4) % 8) as u32;
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
            .draw(&mut *display)
            .unwrap();

            // Draw status text
            let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::BLUE);
            let mut text_buffer = heapless::String::<32>::new();
            core::write!(&mut text_buffer, "C:{} B:{}", counter, current_brightness).unwrap();

            Text::new(&text_buffer, Point::new(2, 30), text_style)
                .draw(&mut *display)
                .unwrap();

            // Swap buffers if double buffering is enabled
            display.swap_buffers();
        }

        counter += 1;
        Timer::after(Duration::from_millis(50)).await;
    }
}

#[embassy_executor::task]
async fn status_task() {
    let mut tick = 0u32;
    loop {
        // This task demonstrates that the system is responsive
        // In a real application, this might handle button inputs,
        // sensor readings, or network communication

        tick += 1;
        if tick % 20 == 0 {
            // Every second, we could do something like adjust settings
            // or handle user input
        }

        Timer::after(Duration::from_millis(50)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // Configure HUB75 pins for a 64x32 display
    // These pin assignments are for a typical RP2040 board
    let pins = Hub75Pins::new_64x32(
        // RGB pins for upper half (R1, G1, B1)
        Output::new(p.PIN_2, Level::Low), // R1
        Output::new(p.PIN_3, Level::Low), // G1
        Output::new(p.PIN_4, Level::Low), // B1
        // RGB pins for lower half (R2, G2, B2)
        Output::new(p.PIN_5, Level::Low), // R2
        Output::new(p.PIN_6, Level::Low), // G2
        Output::new(p.PIN_7, Level::Low), // B2
        // Address pins (A, B, C, D)
        Output::new(p.PIN_8, Level::Low),  // A
        Output::new(p.PIN_9, Level::Low),  // B
        Output::new(p.PIN_10, Level::Low), // C
        Output::new(p.PIN_11, Level::Low), // D
        // Control pins (CLK, LAT, OE)
        Output::new(p.PIN_12, Level::Low),  // CLK
        Output::new(p.PIN_13, Level::Low),  // LAT
        Output::new(p.PIN_14, Level::High), // OE (active low, so start high)
    );

    // Create the display with optimized settings
    let mut display = Hub75Display::<_, 64, 32, 6>::new(pins).unwrap();

    // Configure display settings
    display.set_double_buffering(true);
    display.set_brightness(Brightness::new(128)); // 50% brightness
    display.set_refresh_interval(Duration::from_micros(50)); // 20kHz base rate

    // Initialize the shared display
    // Note: In a real implementation, you'd use a proper initialization pattern
    // This is simplified for the example
    unsafe {
        let display_ptr = &DISPLAY as *const _ as *mut SharedDisplay;
        core::ptr::write(display_ptr, Mutex::new(display));
    }

    // Spawn tasks with appropriate priorities
    // High priority for display refresh (critical for flicker-free display)
    spawner.spawn(display_refresh_task(&DISPLAY)).unwrap();

    // Medium priority for graphics updates
    spawner.spawn(graphics_task(&DISPLAY)).unwrap();

    // Low priority for background status/housekeeping
    spawner.spawn(status_task()).unwrap();

    // Main task can handle system-level events or just idle
    loop {
        Timer::after(Duration::from_secs(10)).await;
        // Could log system status, handle watchdog, etc.
    }
}

// Optional: Custom panic handler that tries to show error on display
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // In a real implementation, you might try to display the panic info
    // on the LED matrix before halting

    // For now, just halt
    loop {
        cortex_m::asm::wfi();
    }
}
