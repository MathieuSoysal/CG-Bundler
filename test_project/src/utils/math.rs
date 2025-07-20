//! Mathematical utilities and algorithms.

use crate::core::Position;

/// Mathematical utility functions.
pub struct MathUtils;

impl MathUtils {
    /// Calculates the Manhattan distance between two points.
    pub fn manhattan_distance(p1: &Position, p2: &Position) -> u32 {
        p1.distance_to(p2)
    }

    /// Calculates the Euclidean distance between two points.
    pub fn euclidean_distance(p1: &Position, p2: &Position) -> f64 {
        let dx = (p1.x as f64) - (p2.x as f64);
        let dy = (p1.y as f64) - (p2.y as f64);
        (dx * dx + dy * dy).sqrt()
    }

    /// Finds the midpoint between two positions.
    pub fn midpoint(p1: &Position, p2: &Position) -> Position {
        Position::new((p1.x + p2.x) / 2, (p1.y + p2.y) / 2)
    }

    /// Clamps a value between min and max.
    pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }

    /// Linear interpolation between two values.
    pub fn lerp(start: f64, end: f64, t: f64) -> f64 {
        start + (end - start) * t
    }

    /// Calculates the area of effect for splash damage.
    pub fn get_positions_in_radius(center: &Position, radius: u32) -> Vec<Position> {
        let mut positions = Vec::new();

        for dx in 0..=radius {
            for dy in 0..=radius {
                if dx + dy <= radius {
                    // Add all four quadrants
                    positions.push(Position::new(center.x + dx, center.y + dy));
                    if dx > 0 {
                        positions.push(Position::new(center.x - dx, center.y + dy));
                    }
                    if dy > 0 {
                        positions.push(Position::new(center.x + dx, center.y - dy));
                    }
                    if dx > 0 && dy > 0 {
                        positions.push(Position::new(center.x - dx, center.y - dy));
                    }
                }
            }
        }

        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean_distance() {
        let p1 = Position::new(0, 0);
        let p2 = Position::new(3, 4);
        assert_eq!(MathUtils::euclidean_distance(&p1, &p2), 5.0);
    }

    #[test]
    fn test_midpoint() {
        let p1 = Position::new(2, 2);
        let p2 = Position::new(6, 8);
        let mid = MathUtils::midpoint(&p1, &p2);
        assert_eq!(mid, Position::new(4, 5));
    }

    #[test]
    fn test_clamp() {
        assert_eq!(MathUtils::clamp(5, 0, 10), 5);
        assert_eq!(MathUtils::clamp(-1, 0, 10), 0);
        assert_eq!(MathUtils::clamp(15, 0, 10), 10);
    }

    #[test]
    fn test_positions_in_radius() {
        let center = Position::new(5, 5);
        let positions = MathUtils::get_positions_in_radius(&center, 1);
        assert!(positions.contains(&Position::new(5, 5))); // Center
        assert!(positions.contains(&Position::new(6, 5))); // Right
        assert!(positions.contains(&Position::new(5, 6))); // Up
        assert!(positions.len() >= 5); // At least center + 4 adjacent
    }
}
