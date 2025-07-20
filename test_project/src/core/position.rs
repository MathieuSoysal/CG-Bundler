//! Position and coordinate utilities for the game grid.

/// Represents a position on the game grid with x,y coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

impl Position {
    /// Creates a new position with the given coordinates.
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    /// Calculates the Manhattan distance to another position.
    pub fn distance_to(&self, other: &Position) -> u32 {
        ((self.x as i32 - other.x as i32).abs() + (self.y as i32 - other.y as i32).abs()) as u32
    }

    /// Returns a new position moved one step towards the target position.
    pub fn move_towards(&self, target: &Position) -> Position {
        let dx = if target.x > self.x {
            1
        } else if target.x < self.x {
            -1
        } else {
            0
        };
        let dy = if target.y > self.y {
            1
        } else if target.y < self.y {
            -1
        } else {
            0
        };

        Position::new((self.x as i32 + dx) as u32, (self.y as i32 + dy) as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_distance() {
        let pos1 = Position::new(0, 0);
        let pos2 = Position::new(3, 4);
        assert_eq!(pos1.distance_to(&pos2), 7);
    }

    #[test]
    fn test_position_move_towards() {
        let pos1 = Position::new(2, 2);
        let target = Position::new(5, 5);
        let next_pos = pos1.move_towards(&target);
        assert_eq!(next_pos, Position::new(3, 3));
    }
}
