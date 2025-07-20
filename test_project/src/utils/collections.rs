//! Collection utilities and data structure helpers.

use crate::core::{Agent, Position};
use std::collections::HashMap;

/// Utility functions for working with collections.
pub struct CollectionUtils;

impl CollectionUtils {
    /// Groups agents by their player ID.
    pub fn group_agents_by_player(agents: &[Agent]) -> HashMap<u32, Vec<&Agent>> {
        let mut groups = HashMap::new();

        for agent in agents {
            groups
                .entry(agent.get_player())
                .or_insert_with(Vec::new)
                .push(agent);
        }

        groups
    }

    /// Finds the closest agent to a target position.
    pub fn find_closest_agent<'a>(agents: &'a [Agent], target: &Position) -> Option<&'a Agent> {
        agents
            .iter()
            .min_by_key(|agent| agent.get_distance_to(target))
    }

    /// Finds all agents within a certain distance from a position.
    pub fn find_agents_in_range<'a>(
        agents: &'a [Agent],
        center: &Position,
        range: u32,
    ) -> Vec<&'a Agent> {
        agents
            .iter()
            .filter(|agent| agent.get_distance_to(center) <= range)
            .collect()
    }

    /// Sorts agents by wetness level (highest first).
    pub fn sort_by_wetness(agents: &mut [Agent]) {
        agents.sort_by(|a, b| b.get_wetness().cmp(&a.get_wetness()));
    }

    /// Filters agents that belong to a specific player.
    pub fn filter_by_player<'a>(agents: &'a [Agent], player_id: u32) -> Vec<&'a Agent> {
        agents
            .iter()
            .filter(|agent| agent.get_player() == player_id)
            .collect()
    }

    /// Creates a position lookup map for fast position-based queries.
    pub fn create_position_map(agents: &[Agent]) -> HashMap<Position, u32> {
        agents
            .iter()
            .map(|agent| (*agent.get_position(), agent.get_agent_id()))
            .collect()
    }
}

/// Priority queue implementation for pathfinding and decision making.
pub struct PriorityQueue<T> {
    items: Vec<(T, u32)>, // (item, priority)
}

impl<T> PriorityQueue<T> {
    /// Creates a new empty priority queue.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Adds an item with the given priority (lower values = higher priority).
    pub fn push(&mut self, item: T, priority: u32) {
        self.items.push((item, priority));
        self.items.sort_by_key(|(_, p)| *p);
    }

    /// Removes and returns the highest priority item.
    pub fn pop(&mut self) -> Option<T> {
        if self.items.is_empty() {
            None
        } else {
            Some(self.items.remove(0).0)
        }
    }

    /// Returns the number of items in the queue.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if the queue is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<T> Default for PriorityQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_agents_by_player() {
        let agents = vec![
            Agent::new(1, 0, 0, 0, 3, 5, 10, 2),
            Agent::new(2, 0, 1, 1, 3, 5, 10, 2),
            Agent::new(3, 1, 2, 2, 3, 5, 10, 2),
        ];

        let groups = CollectionUtils::group_agents_by_player(&agents);
        assert_eq!(groups.get(&0).unwrap().len(), 2);
        assert_eq!(groups.get(&1).unwrap().len(), 1);
    }

    #[test]
    fn test_find_closest_agent() {
        let agents = vec![
            Agent::new(1, 0, 0, 0, 3, 5, 10, 2),
            Agent::new(2, 0, 5, 5, 3, 5, 10, 2),
        ];

        let target = Position::new(1, 1);
        let closest = CollectionUtils::find_closest_agent(&agents, &target);
        assert_eq!(closest.unwrap().get_agent_id(), 1);
    }

    #[test]
    fn test_priority_queue() {
        let mut queue = PriorityQueue::new();

        queue.push("low", 10);
        queue.push("high", 1);
        queue.push("medium", 5);

        assert_eq!(queue.pop(), Some("high"));
        assert_eq!(queue.pop(), Some("medium"));
        assert_eq!(queue.pop(), Some("low"));
        assert_eq!(queue.pop(), None);
    }
}
