//! Arena geometry, tile system, and spatial utilities.

use serde::{Deserialize, Serialize};
use shared::Position;

/// The game arena containing tile layout and dimensions.
///
/// Based on the legacy 18x32 tile system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arena {
    pub width: u32,
    pub height: u32,
    pub tile_size: f32,
    pub tiles: Vec<Vec<TileType>>,
}

impl Arena {
    /// Creates a default arena (18x32 tiles).
    pub fn new() -> Self {
        let width = 18;
        let height = 32;
        let tile_size = 1.0;

        // Initialize arena with grass, river, and bridges
        let mut tiles = vec![vec![TileType::Grass; width as usize]; height as usize];

        // Add river at center (x = 15-16 in legacy 32-wide arena maps to y = 15-16 in our 18-high arena)
        // Our arena is 18 wide x 32 high, so river runs horizontally at y = 15-16
        for x in 0..width as usize {
            tiles[15][x] = TileType::River;
            tiles[16][x] = TileType::River;
        }

        // Add bridges at x = 3 and x = 14 (1 unit wide each)
        for y in 15..=16 {
            tiles[y][3] = TileType::Bridge;
            tiles[y][14] = TileType::Bridge;
        }

        Self {
            width,
            height,
            tile_size,
            tiles,
        }
    }

    /// Gets the tile type at the given position.
    pub fn get_tile(&self, x: u32, y: u32) -> Option<TileType> {
        self.tiles
            .get(y as usize)
            .and_then(|row| row.get(x as usize))
            .copied()
    }

    /// Converts world position to tile coordinates.
    pub fn world_to_tile(&self, pos: &Position) -> (u32, u32) {
        let x = (pos.x / self.tile_size).floor() as u32;
        let y = (pos.y / self.tile_size).floor() as u32;
        (x.min(self.width - 1), y.min(self.height - 1))
    }

    /// Converts tile coordinates to world position (center of tile).
    pub fn tile_to_world(&self, x: u32, y: u32) -> Position {
        Position::new(
            (x as f32 + 0.5) * self.tile_size,
            (y as f32 + 0.5) * self.tile_size,
        )
    }

    /// Checks if a position is within arena bounds.
    pub fn is_in_bounds(&self, pos: &Position) -> bool {
        pos.x >= 0.0
            && pos.y >= 0.0
            && pos.x < self.width as f32 * self.tile_size
            && pos.y < self.height as f32 * self.tile_size
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of tiles in the arena.
///
/// Based on the legacy engine's 6 tile types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Grass,
    Bridge,
    River,
    Tower,
    Decoration,
    Wall,
}

impl TileType {
    /// Returns whether units can walk on this tile.
    pub fn is_walkable(&self) -> bool {
        matches!(self, TileType::Grass | TileType::Bridge | TileType::Tower)
    }
}
