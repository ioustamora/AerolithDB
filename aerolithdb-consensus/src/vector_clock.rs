use std::collections::HashMap;
use serde::{Deserialize, Deserializer, Serialize};

/// Vector clock implementation for distributed consensus
#[derive(Debug, Clone, Serialize)]
pub struct VectorClock<T>
where
    T: Clone + Eq + std::hash::Hash + Serialize,
{
    clocks: HashMap<T, u64>,
}

impl<T> VectorClock<T>
where
    T: Clone + Eq + std::hash::Hash + Serialize,
{
    /// Create a new vector clock
    pub fn new() -> Self {
        Self {
            clocks: HashMap::new(),
        }
    }

    /// Increment the clock for a given node
    pub fn increment(&mut self, node: T) {
        let current = self.clocks.get(&node).unwrap_or(&0);
        self.clocks.insert(node, current + 1);
    }

    /// Get the clock value for a node
    pub fn get(&self, node: &T) -> u64 {
        self.clocks.get(node).copied().unwrap_or(0)
    }

    /// Merge with another vector clock (take maximum values)
    pub fn merge(&mut self, other: &VectorClock<T>) {
        for (node, clock) in &other.clocks {
            let current = self.clocks.get(node).unwrap_or(&0);
            self.clocks.insert(node.clone(), (*current).max(*clock));
        }
    }

    /// Check if this clock happens before another
    pub fn happens_before(&self, other: &VectorClock<T>) -> bool {
        let mut strictly_less = false;
        
        // Check all nodes in our clock
        for (node, our_clock) in &self.clocks {
            let their_clock = other.clocks.get(node).unwrap_or(&0);
            if our_clock > their_clock {
                return false; // Not happening before
            }
            if our_clock < their_clock {
                strictly_less = true;
            }
        }

        // Check nodes only in their clock
        for (node, their_clock) in &other.clocks {
            if !self.clocks.contains_key(node) && *their_clock > 0 {
                strictly_less = true;
            }
        }

        strictly_less
    }

    /// Check if this clock is concurrent with another
    pub fn concurrent(&self, other: &VectorClock<T>) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }

    /// Check if this clock is equal to another
    pub fn equals(&self, other: &VectorClock<T>) -> bool {
        // Get all unique nodes from both clocks
        let mut all_nodes = std::collections::HashSet::new();
        all_nodes.extend(self.clocks.keys());
        all_nodes.extend(other.clocks.keys());

        // Check if all nodes have the same value
        for node in all_nodes {
            let our_value = self.clocks.get(node).unwrap_or(&0);
            let their_value = other.clocks.get(node).unwrap_or(&0);
            if our_value != their_value {
                return false;
            }
        }

        true
    }

    /// Update the clock based on a received message
    pub fn update_on_receive(&mut self, sender: T, received_clock: &VectorClock<T>) {
        // Merge with received clock
        self.merge(received_clock);
        
        // Increment our own clock
        self.increment(sender);
    }

    /// Get all nodes in the clock
    pub fn nodes(&self) -> Vec<&T> {
        self.clocks.keys().collect()
    }

    /// Get the sum of all clock values (logical time)
    pub fn logical_time(&self) -> u64 {
        self.clocks.values().sum()
    }

    /// Create a snapshot of the current clock
    pub fn snapshot(&self) -> VectorClock<T> {
        self.clone()
    }

    /// Reset the clock
    pub fn reset(&mut self) {
        self.clocks.clear();
    }

    /// Remove a node from the clock
    pub fn remove_node(&mut self, node: &T) -> Option<u64> {
        self.clocks.remove(node)
    }

    /// Check if the clock is empty
    pub fn is_empty(&self) -> bool {
        self.clocks.is_empty()
    }

    /// Get the number of nodes tracked
    pub fn node_count(&self) -> usize {
        self.clocks.len()
    }

    /// Get the maximum clock value
    pub fn max_value(&self) -> u64 {
        self.clocks.values().copied().max().unwrap_or(0)
    }

    /// Compare two vector clocks and return ordering
    pub fn compare(&self, other: &VectorClock<T>) -> VectorClockOrdering {
        if self.equals(other) {
            VectorClockOrdering::Equal
        } else if self.happens_before(other) {
            VectorClockOrdering::Before
        } else if other.happens_before(self) {
            VectorClockOrdering::After
        } else {
            VectorClockOrdering::Concurrent
        }
    }
}

/// Ordering relationship between vector clocks
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VectorClockOrdering {
    Before,
    After,
    Equal,
    Concurrent,
}

impl<T> Default for VectorClock<T>
where
    T: Clone + Eq + std::hash::Hash + Serialize,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> std::fmt::Display for VectorClock<T>
where
    T: std::fmt::Display + Clone + Eq + std::hash::Hash + Serialize,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        let mut first = true;
        for (node, clock) in &self.clocks {
            if !first {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", node, clock)?;
            first = false;
        }
        write!(f, "}}")
    }
}

// Manual implementation of Deserialize to avoid trait bound conflicts
impl<'de, T> Deserialize<'de> for VectorClock<T>
where
    T: Clone + Eq + std::hash::Hash + Serialize + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let clocks = HashMap::<T, u64>::deserialize(deserializer)?;
        Ok(VectorClock { clocks })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock_basic_operations() {
        let mut clock = VectorClock::new();
        
        // Test increment
        clock.increment("node1".to_string());
        assert_eq!(clock.get(&"node1".to_string()), 1);
        
        clock.increment("node1".to_string());
        assert_eq!(clock.get(&"node1".to_string()), 2);
        
        clock.increment("node2".to_string());
        assert_eq!(clock.get(&"node2".to_string()), 1);
        assert_eq!(clock.get(&"node1".to_string()), 2);
    }

    #[test]
    fn test_vector_clock_happens_before() {
        let mut clock1 = VectorClock::new();
        let mut clock2 = VectorClock::new();
        
        clock1.increment("node1".to_string());
        clock2.increment("node1".to_string());
        clock2.increment("node1".to_string());
        
        assert!(clock1.happens_before(&clock2));
        assert!(!clock2.happens_before(&clock1));
    }

    #[test]
    fn test_vector_clock_concurrent() {
        let mut clock1 = VectorClock::new();
        let mut clock2 = VectorClock::new();
        
        clock1.increment("node1".to_string());
        clock2.increment("node2".to_string());
        
        assert!(clock1.concurrent(&clock2));
        assert!(clock2.concurrent(&clock1));
    }

    #[test]
    fn test_vector_clock_merge() {
        let mut clock1 = VectorClock::new();
        let mut clock2 = VectorClock::new();
        
        clock1.increment("node1".to_string());
        clock1.increment("node1".to_string());
        clock1.increment("node2".to_string());
        
        clock2.increment("node1".to_string());
        clock2.increment("node3".to_string());
        
        clock1.merge(&clock2);
        
        assert_eq!(clock1.get(&"node1".to_string()), 2); // max(2, 1)
        assert_eq!(clock1.get(&"node2".to_string()), 1); // max(1, 0)
        assert_eq!(clock1.get(&"node3".to_string()), 1); // max(0, 1)
    }
}
