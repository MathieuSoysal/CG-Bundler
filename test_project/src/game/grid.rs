//! Grid and tile management for the game world.

use crate::core::{position::Position, types::TileType};

impl From<u32> for TileType {
    fn from(value: u32) -> Self {
        match value {
            0 => TileType::Empty,
            1 => TileType::LowCover,
            2 => TileType::HighCover,
            _ => TileType::Empty,
        }
    }
}

/// Represents a single tile on the game grid.
#[derive(Debug, Clone)]
pub struct Tile {
    x: u32,
    y: u32,
    tile_type: TileType,
}

impl Tile {
    /// Creates a new tile with the specified position and type.
    pub fn new(x: u32, y: u32, tile_type: TileType) -> Self {
        Self { x, y, tile_type }
    }

    /// Returns the position of this tile.
    pub fn get_position(&self) -> Position {
        Position::new(self.x, self.y)
    }

    /// Returns true if this tile provides cover for units.
    pub fn provides_cover(&self) -> bool {
        matches!(self.tile_type, TileType::LowCover | TileType::HighCover)
    }

    /// Returns the tile type.
    pub fn get_tile_type(&self) -> TileType {
        self.tile_type
    }
}

/// Manages the 2D grid of tiles that make up the game world.
#[derive(Debug)]
pub struct Grid {
    width: u32,
    height: u32,
    tiles: Vec<Vec<Tile>>,
}

impl Grid {
    /// Creates a new grid with the specified dimensions, filled with empty tiles.
    pub fn new(width: u32, height: u32) -> Self {
        let tiles = (0..height)
            .map(|y| {
                (0..width)
                    .map(|x| Tile::new(x, y, TileType::Empty))
                    .collect()
            })
            .collect();

        Self {
            width,
            height,
            tiles,
        }
    }

    /// Returns a reference to the tile at the given coordinates, if valid.
    pub fn get_tile(&self, x: u32, y: u32) -> Option<&Tile> {
        if self.is_valid_position(x, y) {
            Some(&self.tiles[y as usize][x as usize])
        } else {
            None
        }
    }

    /// Checks if the given coordinates are within the grid bounds.
    pub fn is_valid_position(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }

    /// Returns all valid neighboring positions for the given coordinates.
    pub fn get_neighbors(&self, x: u32, y: u32) -> Vec<Position> {
        let mut neighbors = Vec::new();
        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];

        for (dx, dy) in directions.iter() {
            let new_x = x as i32 + dx;
            let new_y = y as i32 + dy;

            if new_x >= 0 && new_y >= 0 {
                let new_x = new_x as u32;
                let new_y = new_y as u32;
                if self.is_valid_position(new_x, new_y) {
                    neighbors.push(Position::new(new_x, new_y));
                }
            }
        }

        neighbors
    }

    /// Updates the tile type at the given coordinates.
    pub fn set_tile(&mut self, x: u32, y: u32, tile_type: TileType) {
        if self.is_valid_position(x, y) {
            self.tiles[y as usize][x as usize] = Tile::new(x, y, tile_type);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_valid_position() {
        let grid = Grid::new(10, 10);
        assert!(grid.is_valid_position(5, 5));
        assert!(!grid.is_valid_position(15, 5));
    }

    #[test]
    fn test_grid_neighbors() {
        let grid = Grid::new(10, 10);
        let neighbors = grid.get_neighbors(5, 5);

        // Should have 4 neighbors for a center position
        assert_eq!(neighbors.len(), 4);

        // Test corner position
        let corner_neighbors = grid.get_neighbors(0, 0);
        assert_eq!(corner_neighbors.len(), 2);
    }

    #[test]
    fn test_tile_cover() {
        let empty_tile = Tile::new(0, 0, TileType::Empty);
        let low_cover_tile = Tile::new(1, 1, TileType::LowCover);
        let high_cover_tile = Tile::new(2, 2, TileType::HighCover);

        assert!(!empty_tile.provides_cover());
        assert!(low_cover_tile.provides_cover());
        assert!(high_cover_tile.provides_cover());
    }
}
