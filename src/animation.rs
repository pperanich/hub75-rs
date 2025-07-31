//! Animation support for HUB75 displays, inspired by microbit patterns

use crate::{
    color::Hub75Color,
    error::{AnimationError, Hub75Error},
    frame_buffer::Hub75FrameBuffer,
};
use embassy_time::{Duration, Instant};


/// Animation effects that can be applied
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AnimationEffect {
    /// No effect - frames are displayed as-is
    None,
    /// Sliding effect - frames slide in from the right
    Slide,
    /// Fade effect - frames fade in and out
    Fade,
    /// Wipe effect - frames are revealed column by column
    Wipe,
}

/// Current state of an animation
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AnimationState<const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize> {
    /// Animation is waiting for the next frame time
    Wait,
    /// Apply the given frame to the display
    Apply(Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>),
    /// Animation is complete
    Done,
}

/// Animation data source
pub enum AnimationData<'a, const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize> {
    /// Array of frame buffers
    Frames(&'a [Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>]),
    /// Raw RGB data (width * height * 3 bytes per frame)
    RgbData(&'a [u8]),
    /// Text data to be converted to frames
    Text(&'a str),
}

impl<'a, const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize> 
    AnimationData<'a, WIDTH, HEIGHT, COLOR_BITS> 
{
    /// Get the number of frames in the animation data
    pub fn frame_count(&self) -> usize {
        match self {
            AnimationData::Frames(frames) => frames.len(),
            AnimationData::RgbData(data) => data.len() / (WIDTH * HEIGHT * 3),
            AnimationData::Text(text) => text.len(), // One frame per character
        }
    }

    /// Get a specific frame from the animation data
    pub fn get_frame(&self, index: usize) -> Result<Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>, Hub75Error> {
        match self {
            AnimationData::Frames(frames) => {
                if index < frames.len() {
                    Ok(frames[index].clone())
                } else {
                    Err(Hub75Error::AnimationError(AnimationError::InvalidData))
                }
            }
            AnimationData::RgbData(data) => {
                let frame_size = WIDTH * HEIGHT * 3;
                let start = index * frame_size;
                let end = start + frame_size;
                
                if end <= data.len() {
                    Hub75FrameBuffer::from_rgb_data(&data[start..end])
                } else {
                    Err(Hub75Error::AnimationError(AnimationError::InvalidData))
                }
            }
            AnimationData::Text(text) => {
                // For text, create a frame with the character at the given index
                if index < text.len() {
                    let mut frame = Hub75FrameBuffer::new();
                    // This is a simplified implementation - in practice you'd want
                    // to use a font renderer here
                    let char_byte = text.as_bytes()[index];
                    self.render_character_to_frame(&mut frame, char_byte)?;
                    Ok(frame)
                } else {
                    Err(Hub75Error::AnimationError(AnimationError::InvalidData))
                }
            }
        }
    }

    /// Render a character to a frame buffer (simplified implementation)
    fn render_character_to_frame(&self, frame: &mut Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>, char_byte: u8) -> Result<(), Hub75Error> {
        // This is a very basic 5x5 font implementation
        // In a real implementation, you'd use a proper font library
        let pattern = match char_byte {
            b'A' => [
                0b01110,
                0b10001,
                0b10001,
                0b11111,
                0b10001,
            ],
            b'B' => [
                0b11110,
                0b10001,
                0b11110,
                0b10001,
                0b11110,
            ],
            b'C' => [
                0b01111,
                0b10000,
                0b10000,
                0b10000,
                0b01111,
            ],
            _ => [
                0b11111,
                0b10001,
                0b10001,
                0b10001,
                0b11111,
            ], // Default pattern
        };

        let color = Hub75Color::white();
        let start_x = (WIDTH - 5) / 2; // Center horizontally
        let start_y = (HEIGHT - 5) / 2; // Center vertically

        for (y, &row) in pattern.iter().enumerate() {
            for x in 0..5 {
                if (row >> (4 - x)) & 1 == 1 {
                    if start_x + x < WIDTH && start_y + y < HEIGHT {
                        frame.set_pixel(start_x + x, start_y + y, color)?;
                    }
                }
            }
        }

        Ok(())
    }
}

/// Animation controller
pub struct Animation<'a, const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize> {
    /// Animation data source
    data: AnimationData<'a, WIDTH, HEIGHT, COLOR_BITS>,
    /// Current frame index
    frame_index: usize,
    /// Current sequence position (for effects like sliding)
    sequence: usize,
    /// Current step in the animation
    step: usize,
    /// Total number of steps in the animation
    total_steps: usize,
    /// Animation effect to apply
    effect: AnimationEffect,
    /// Duration between steps
    step_duration: Duration,
    /// Time when the next step should occur
    next_step_time: Instant,
}

impl<'a, const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize> 
    Animation<'a, WIDTH, HEIGHT, COLOR_BITS> 
{
    /// Create a new animation
    pub fn new(
        data: AnimationData<'a, WIDTH, HEIGHT, COLOR_BITS>,
        effect: AnimationEffect,
        total_duration: Duration,
    ) -> Result<Self, AnimationError> {
        let frame_count = data.frame_count();
        if frame_count == 0 {
            return Err(AnimationError::InvalidData);
        }

        let total_steps = match effect {
            AnimationEffect::None => frame_count,
            AnimationEffect::Slide => frame_count * WIDTH,
            AnimationEffect::Fade => frame_count * 16, // 16 fade steps per frame
            AnimationEffect::Wipe => frame_count * WIDTH,
        };

        let step_duration = total_duration.checked_div(total_steps as u32)
            .ok_or(AnimationError::TooFast)?;

        Ok(Self {
            data,
            frame_index: 0,
            sequence: 0,
            step: 0,
            total_steps,
            effect,
            step_duration,
            next_step_time: Instant::now(),
        })
    }

    /// Get the next animation state
    pub fn next(&mut self, now: Instant) -> AnimationState<WIDTH, HEIGHT, COLOR_BITS> {
        if self.step >= self.total_steps {
            return AnimationState::Done;
        }

        if now < self.next_step_time {
            return AnimationState::Wait;
        }

        // Generate the current frame based on the effect
        let frame = match self.generate_current_frame() {
            Ok(frame) => frame,
            Err(_) => return AnimationState::Done,
        };

        // Advance to the next step
        self.advance_step();
        self.next_step_time += self.step_duration;

        AnimationState::Apply(frame)
    }

    /// Generate the current frame based on the effect and current state
    fn generate_current_frame(&self) -> Result<Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>, Hub75Error> {
        match self.effect {
            AnimationEffect::None => {
                self.data.get_frame(self.frame_index)
            }
            AnimationEffect::Slide => {
                self.generate_slide_frame()
            }
            AnimationEffect::Fade => {
                self.generate_fade_frame()
            }
            AnimationEffect::Wipe => {
                self.generate_wipe_frame()
            }
        }
    }

    /// Generate a frame for the slide effect
    fn generate_slide_frame(&self) -> Result<Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>, Hub75Error> {
        let mut result = Hub75FrameBuffer::new();
        
        // Get current and next frames
        let current_frame = self.data.get_frame(self.frame_index)?;
        let next_frame = if self.frame_index + 1 < self.data.frame_count() {
            self.data.get_frame(self.frame_index + 1)?
        } else {
            Hub75FrameBuffer::new() // Black frame
        };

        // Slide effect: current frame slides left, next frame slides in from right
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let pixel = if x + self.sequence < WIDTH {
                    // Show current frame, shifted left
                    current_frame.get_pixel(x + self.sequence, y).unwrap_or(Hub75Color::black())
                } else {
                    // Show next frame, coming in from right
                    let next_x = x + self.sequence - WIDTH;
                    next_frame.get_pixel(next_x, y).unwrap_or(Hub75Color::black())
                };
                
                result.set_pixel(x, y, pixel)?;
            }
        }

        Ok(result)
    }

    /// Generate a frame for the fade effect
    fn generate_fade_frame(&self) -> Result<Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>, Hub75Error> {
        let mut result = Hub75FrameBuffer::new();
        let frame = self.data.get_frame(self.frame_index)?;
        
        // Calculate fade factor (0-15)
        let fade_step = self.sequence;
        let fade_factor = if fade_step < 8 {
            fade_step // Fade in
        } else {
            15 - fade_step // Fade out
        };

        // Apply fade to each pixel
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let original = frame.get_pixel(x, y)?;
                let faded = Hub75Color::new(
                    (original.r * fade_factor as u8) / 15,
                    (original.g * fade_factor as u8) / 15,
                    (original.b * fade_factor as u8) / 15,
                );
                result.set_pixel(x, y, faded)?;
            }
        }

        Ok(result)
    }

    /// Generate a frame for the wipe effect
    fn generate_wipe_frame(&self) -> Result<Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>, Hub75Error> {
        let mut result = Hub75FrameBuffer::new();
        let frame = self.data.get_frame(self.frame_index)?;
        
        // Wipe effect: reveal columns from left to right
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if x <= self.sequence {
                    let pixel = frame.get_pixel(x, y)?;
                    result.set_pixel(x, y, pixel)?;
                }
                // Pixels beyond the wipe position remain black (default)
            }
        }

        Ok(result)
    }

    /// Advance to the next step in the animation
    fn advance_step(&mut self) {
        self.step += 1;
        
        match self.effect {
            AnimationEffect::None => {
                self.frame_index = self.step;
            }
            AnimationEffect::Slide => {
                self.sequence += 1;
                if self.sequence >= WIDTH {
                    self.sequence = 0;
                    self.frame_index += 1;
                }
            }
            AnimationEffect::Fade => {
                self.sequence += 1;
                if self.sequence >= 16 {
                    self.sequence = 0;
                    self.frame_index += 1;
                }
            }
            AnimationEffect::Wipe => {
                self.sequence += 1;
                if self.sequence >= WIDTH {
                    self.sequence = 0;
                    self.frame_index += 1;
                }
            }
        }
    }

    /// Check if the animation is complete
    pub fn is_done(&self) -> bool {
        self.step >= self.total_steps
    }

    /// Reset the animation to the beginning
    pub fn reset(&mut self) {
        self.frame_index = 0;
        self.sequence = 0;
        self.step = 0;
        self.next_step_time = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_creation() {
        let frames = [
            Hub75FrameBuffer::<32, 16, 6>::new(),
            Hub75FrameBuffer::<32, 16, 6>::new(),
        ];
        
        let animation = Animation::new(
            AnimationData::Frames(&frames),
            AnimationEffect::None,
            Duration::from_secs(1),
        );
        
        assert!(animation.is_ok());
        let animation = animation.unwrap();
        assert!(!animation.is_done());
    }

    #[test]
    fn test_animation_data_frame_count() {
        let frames = [
            Hub75FrameBuffer::<32, 16, 6>::new(),
            Hub75FrameBuffer::<32, 16, 6>::new(),
            Hub75FrameBuffer::<32, 16, 6>::new(),
        ];
        
        let data = AnimationData::Frames(&frames);
        assert_eq!(data.frame_count(), 3);
        
        let text_data = AnimationData::<32, 16, 6>::Text("Hello");
        assert_eq!(text_data.frame_count(), 5);
        
        // RGB data: 32 * 16 * 3 = 1536 bytes per frame
        let rgb_data = vec![0u8; 1536 * 2]; // 2 frames
        let rgb_animation_data = AnimationData::<32, 16, 6>::RgbData(&rgb_data);
        assert_eq!(rgb_animation_data.frame_count(), 2);
    }

    #[test]
    fn test_animation_effects() {
        let frames = [Hub75FrameBuffer::<32, 16, 6>::new()];
        
        // Test different effects create different step counts
        let none_anim = Animation::new(
            AnimationData::Frames(&frames),
            AnimationEffect::None,
            Duration::from_secs(1),
        ).unwrap();
        
        let slide_anim = Animation::new(
            AnimationData::Frames(&frames),
            AnimationEffect::Slide,
            Duration::from_secs(1),
        ).unwrap();
        
        assert!(slide_anim.total_steps > none_anim.total_steps);
    }
}