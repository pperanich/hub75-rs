//! Error types for the HUB75 driver

/// Errors that can occur when using the HUB75 driver
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Hub75Error {
    /// Pin operation failed
    PinError,
    /// Invalid coordinates provided
    InvalidCoordinates,
    /// Invalid color value
    InvalidColor,
    /// Animation error
    AnimationError(AnimationError),
    /// Buffer overflow
    BufferOverflow,
}

/// Animation-specific errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AnimationError {
    /// Animation is too fast for the refresh rate
    TooFast,
    /// Invalid animation data
    InvalidData,
    /// Animation duration is invalid
    InvalidDuration,
}

impl From<AnimationError> for Hub75Error {
    fn from(err: AnimationError) -> Self {
        Hub75Error::AnimationError(err)
    }
}