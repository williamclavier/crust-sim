//! Shared data structures and utilities used across the engine.



use serde::{Deserialize, Serialize};

/// Represents a 2D position in the arena.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Returns the direction vector (normalized) from this position to another.
    /// Returns (0, 0) if positions are the same.
    pub fn direction_to(&self, other: &Position) -> (f32, f32) {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance < 0.001 {
            (0.0, 0.0)
        } else {
            (dx / distance, dy / distance)
        }
    }

    /// Checks collision between a circle (centered at self with given radius) and a rectangle.
    /// Rectangle is defined by center position, half-width, and half-height.
    /// Returns true if the circle overlaps with the rectangle.
    pub fn circle_collides_rect(&self, circle_radius: f32, rect_center: &Position, rect_half_width: f32, rect_half_height: f32) -> bool {
        // Find the closest point on the rectangle to the circle center
        let closest_x = self.x.max(rect_center.x - rect_half_width).min(rect_center.x + rect_half_width);
        let closest_y = self.y.max(rect_center.y - rect_half_height).min(rect_center.y + rect_half_height);

        // Calculate distance from circle center to this closest point
        let dx = self.x - closest_x;
        let dy = self.y - closest_y;
        let distance_squared = dx * dx + dy * dy;

        // Collision occurs if distance is less than circle radius
        distance_squared <= (circle_radius * circle_radius)
    }
}

/// Represents a 2D velocity vector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Player identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerId {
    Player1,
    Player2,
}

impl PlayerId {
    pub fn opponent(&self) -> Self {
        match self {
            PlayerId::Player1 => PlayerId::Player2,
            PlayerId::Player2 => PlayerId::Player1,
        }
    }
}

/// Common result type for engine operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for the engine.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid action: {0}")]
    InvalidAction(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Configuration(String),
}


pub mod cr_state;

pub use cr_state::{CRState, Tower, Unit, LegalMasks};