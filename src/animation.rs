//! Animation support for HUB75 displays with frame-based timing
//!
//! This module provides a flexible animation system that works with any async runtime.
//! Instead of using time-based animations, it uses frame-based timing which gives
//! more predictable results and better performance.
//!
//! # Key Concepts
//!
//! - **Frame-based timing**: Animations advance based on frame count rather than elapsed time
//! - **Effect system**: Different visual effects can be applied to frame transitions
//! - **Generic runtime**: Works with Embassy, RTIC, or any async runtime
//!
//! # Animation Effects
//!
//! - `None`: Direct frame display without transitions
//! - `Fade`: Smooth fade between frames
//! - `Slide`: Sliding transition effects
//! - `Scroll`: Scrolling text or image effects
//!
//! # Examples
//!
//! ```rust,no_run
//! use hub75::{Animation, AnimationData, AnimationEffect, Hub75FrameBuffer};
//!
//! # fn example() -> Result<(), hub75::AnimationError> {
//! // Create some frames
//! let frames = [
//!     Hub75FrameBuffer::<64, 32, 6>::new(),
//!     Hub75FrameBuffer::<64, 32, 6>::new(),
//! ];
//!
//! // Create an animation with fade effect over 120 frames
//! let mut animation = Animation::new(
//!     AnimationData::Frames(&frames),
//!     AnimationEffect::Fade,
//!     120, // Total frames for the animation
//! )?;
//!
//! // Advance the animation frame by frame
//! loop {
//!     match animation.next() {
//!         hub75::AnimationState::Apply(frame) => {
//!             // Display this frame
//!             break;
//!         }
//!         hub75::AnimationState::Wait => {
//!             // Continue to next frame
//!             continue;
//!         }
//!         hub75::AnimationState::Done => {
//!             // Animation finished
//!             break;
//!         }
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use crate::{color::Hub75Color, frame_buffer::Hub75FrameBuffer, AnimationError, Hub75Error};

/// Trait for animation effects
pub trait AnimationEffectTrait<const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize> {
    /// Apply the effect to generate a frame
    fn apply_effect(
        &self,
        current_frame: &Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>,
        next_frame: Option<&Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>>,
        progress: usize,
        total_steps: usize,
    ) -> Result<Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>, Hub75Error>;

    /// Get the total number of steps for this effect with the given frame count
    fn total_steps(&self, frame_count: usize) -> usize;
}

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

impl<const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize>
    AnimationEffectTrait<WIDTH, HEIGHT, COLOR_BITS> for AnimationEffect
{
    fn apply_effect(
        &self,
        current_frame: &Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>,
        next_frame: Option<&Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>>,
        progress: usize,
        _total_steps: usize,
    ) -> Result<Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>, Hub75Error> {
        match self {
            AnimationEffect::None => Ok(current_frame.clone()),
            AnimationEffect::Slide => self.apply_slide_effect(current_frame, next_frame, progress),
            AnimationEffect::Fade => self.apply_fade_effect(current_frame, progress),
            AnimationEffect::Wipe => self.apply_wipe_effect(current_frame, progress),
        }
    }

    fn total_steps(&self, frame_count: usize) -> usize {
        match self {
            AnimationEffect::None => frame_count,
            AnimationEffect::Slide => frame_count * WIDTH,
            AnimationEffect::Fade => frame_count * 16,
            AnimationEffect::Wipe => frame_count * WIDTH,
        }
    }
}

impl AnimationEffect {
    /// Apply slide effect
    fn apply_slide_effect<const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize>(
        &self,
        current_frame: &Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>,
        next_frame: Option<&Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>>,
        sequence: usize,
    ) -> Result<Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>, Hub75Error> {
        let mut result = Hub75FrameBuffer::new();
        let next = next_frame.cloned().unwrap_or_else(Hub75FrameBuffer::new);

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let pixel = if x + sequence < WIDTH {
                    current_frame
                        .get_pixel(x + sequence, y)
                        .unwrap_or(Hub75Color::black())
                } else {
                    let next_x = x + sequence - WIDTH;
                    next.get_pixel(next_x, y).unwrap_or(Hub75Color::black())
                };
                result.set_pixel(x, y, pixel)?;
            }
        }
        Ok(result)
    }

    /// Apply fade effect
    fn apply_fade_effect<const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize>(
        &self,
        current_frame: &Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>,
        sequence: usize,
    ) -> Result<Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>, Hub75Error> {
        let mut result = Hub75FrameBuffer::new();
        let fade_factor = if sequence < 8 {
            sequence
        } else {
            15 - sequence
        };

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let original = current_frame.get_pixel(x, y)?;
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

    /// Apply wipe effect
    fn apply_wipe_effect<const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize>(
        &self,
        current_frame: &Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>,
        sequence: usize,
    ) -> Result<Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>, Hub75Error> {
        let mut result = Hub75FrameBuffer::new();

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if x <= sequence {
                    let pixel = current_frame.get_pixel(x, y)?;
                    result.set_pixel(x, y, pixel)?;
                }
            }
        }
        Ok(result)
    }
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
    pub fn get_frame(
        &self,
        index: usize,
    ) -> Result<Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>, Hub75Error> {
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
    fn render_character_to_frame(
        &self,
        frame: &mut Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>,
        char_byte: u8,
    ) -> Result<(), Hub75Error> {
        // This is a very basic 5x5 font implementation
        // In a real implementation, you'd use a proper font library
        let pattern = match char_byte {
            b'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001],
            b'B' => [0b11110, 0b10001, 0b11110, 0b10001, 0b11110],
            b'C' => [0b01111, 0b10000, 0b10000, 0b10000, 0b01111],
            _ => [0b11111, 0b10001, 0b10001, 0b10001, 0b11111], // Default pattern
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
    /// Number of frames between steps
    frames_per_step: usize,
    /// Current frame counter
    frame_counter: usize,
}

impl<'a, const WIDTH: usize, const HEIGHT: usize, const COLOR_BITS: usize>
    Animation<'a, WIDTH, HEIGHT, COLOR_BITS>
{
    /// Create a new animation
    pub fn new(
        data: AnimationData<'a, WIDTH, HEIGHT, COLOR_BITS>,
        effect: AnimationEffect,
        total_frames: usize,
    ) -> Result<Self, AnimationError> {
        let frame_count = data.frame_count();
        if frame_count == 0 {
            return Err(AnimationError::InvalidData);
        }

        let total_steps =
            <AnimationEffect as AnimationEffectTrait<WIDTH, HEIGHT, COLOR_BITS>>::total_steps(
                &effect,
                frame_count,
            );
        let frames_per_step = total_frames / total_steps.max(1);

        Ok(Self {
            data,
            frame_index: 0,
            sequence: 0,
            step: 0,
            total_steps,
            effect,
            frames_per_step,
            frame_counter: 0,
        })
    }

    /// Get the next animation state
    pub fn next(&mut self) -> AnimationState<WIDTH, HEIGHT, COLOR_BITS> {
        if self.step >= self.total_steps {
            return AnimationState::Done;
        }

        self.frame_counter += 1;
        if self.frame_counter < self.frames_per_step {
            return AnimationState::Wait;
        }

        // Reset frame counter for next step
        self.frame_counter = 0;

        // Generate the current frame based on the effect
        let frame = match self.generate_current_frame() {
            Ok(frame) => frame,
            Err(_) => return AnimationState::Done,
        };

        // Advance to the next step
        self.advance_step();

        AnimationState::Apply(frame)
    }

    /// Generate the current frame based on the effect and current state
    fn generate_current_frame(
        &self,
    ) -> Result<Hub75FrameBuffer<WIDTH, HEIGHT, COLOR_BITS>, Hub75Error> {
        let current_frame = self.data.get_frame(self.frame_index)?;
        let next_frame = if self.frame_index + 1 < self.data.frame_count() {
            Some(self.data.get_frame(self.frame_index + 1)?)
        } else {
            None
        };

        <AnimationEffect as AnimationEffectTrait<WIDTH, HEIGHT, COLOR_BITS>>::apply_effect(
            &self.effect,
            &current_frame,
            next_frame.as_ref(),
            self.sequence,
            self.total_steps,
        )
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
        self.frame_counter = 0;
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
            60, // 60 frames total
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
            60, // 60 frames total
        )
        .unwrap();

        let slide_anim = Animation::new(
            AnimationData::Frames(&frames),
            AnimationEffect::Slide,
            60, // 60 frames total
        )
        .unwrap();

        assert!(slide_anim.total_steps > none_anim.total_steps);
    }
}
