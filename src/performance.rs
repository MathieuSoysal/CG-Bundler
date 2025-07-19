//! Performance tracking implementation for monitoring execution times

use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::traits::PerformanceTracker;

/// Metrics collector for tracking operation performance
pub struct MetricsCollector {
    timers: HashMap<String, Instant>,
    metrics: HashMap<String, Duration>,
    enabled: bool,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            timers: HashMap::new(),
            metrics: HashMap::new(),
            enabled: true,
        }
    }
    
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    
    /// Get the total processing time across all operations
    pub fn get_total_processing_time(&self) -> Duration {
        self.metrics.values().sum()
    }
    
    /// Get the duration for a specific operation
    pub fn get_operation_time(&self, operation: &str) -> Option<Duration> {
        self.metrics.get(operation).copied()
    }
    
    /// Get all collected metrics
    pub fn get_all_metrics(&self) -> &HashMap<String, Duration> {
        &self.metrics
    }
    
    /// Clear all collected metrics
    pub fn clear(&mut self) {
        self.timers.clear();
        self.metrics.clear();
    }
}

impl PerformanceTracker for MetricsCollector {
    fn start_timer(&mut self, operation: &str) {
        if self.enabled {
            self.timers.insert(operation.to_string(), Instant::now());
        }
    }
    
    fn end_timer(&mut self, operation: &str) {
        if !self.enabled {
            return;
        }
        
        if let Some(start_time) = self.timers.remove(operation) {
            let duration = start_time.elapsed();
            self.metrics.insert(operation.to_string(), duration);
        }
    }
    
    fn report_metrics(&self) {
        if !self.enabled || self.metrics.is_empty() {
            return;
        }
        
        println!("\n📊 Performance Metrics:");
        println!("{}", "─".repeat(50));
        
        let mut operations: Vec<_> = self.metrics.iter().collect();
        operations.sort_by_key(|(_, duration)| *duration);
        operations.reverse(); // Show slowest first
        
        for (operation, duration) in operations {
            println!("  {:<30} {:>8.2}ms", operation, duration.as_secs_f64() * 1000.0);
        }
        
        let total = self.get_total_processing_time();
        println!("{}", "─".repeat(50));
        println!("  {:<30} {:>8.2}ms", "Total", total.as_secs_f64() * 1000.0);
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// No-op performance tracker for production use when metrics are disabled
pub struct NoOpPerformanceTracker;

impl PerformanceTracker for NoOpPerformanceTracker {
    fn start_timer(&mut self, _operation: &str) {
        // Do nothing
    }
    
    fn end_timer(&mut self, _operation: &str) {
        // Do nothing
    }
    
    fn report_metrics(&self) {
        // Do nothing
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_timer_basic_functionality() {
        let mut tracker = MetricsCollector::new();
        
        tracker.start_timer("test_operation");
        thread::sleep(StdDuration::from_millis(10));
        tracker.end_timer("test_operation");
        
        let duration = tracker.get_operation_time("test_operation").unwrap();
        assert!(duration >= StdDuration::from_millis(10));
        assert!(duration < StdDuration::from_millis(100)); // Should be reasonable
    }
    
    #[test]
    fn test_multiple_operations() {
        let mut tracker = MetricsCollector::new();
        
        tracker.start_timer("operation1");
        thread::sleep(StdDuration::from_millis(5));
        tracker.end_timer("operation1");
        
        tracker.start_timer("operation2");
        thread::sleep(StdDuration::from_millis(5));
        tracker.end_timer("operation2");
        
        assert!(tracker.get_operation_time("operation1").is_some());
        assert!(tracker.get_operation_time("operation2").is_some());
        
        let total = tracker.get_total_processing_time();
        assert!(total >= StdDuration::from_millis(10));
    }
    
    #[test]
    fn test_end_timer_without_start() {
        let mut tracker = MetricsCollector::new();
        
        // Should not panic
        tracker.end_timer("nonexistent_operation");
        
        assert!(tracker.get_operation_time("nonexistent_operation").is_none());
    }
    
    #[test]
    fn test_disabled_tracker() {
        let mut tracker = MetricsCollector::new().with_enabled(false);
        
        tracker.start_timer("test_operation");
        thread::sleep(StdDuration::from_millis(10));
        tracker.end_timer("test_operation");
        
        assert!(tracker.get_operation_time("test_operation").is_none());
        assert_eq!(tracker.get_total_processing_time(), StdDuration::ZERO);
    }
    
    #[test]
    fn test_clear_metrics() {
        let mut tracker = MetricsCollector::new();
        
        tracker.start_timer("test_operation");
        tracker.end_timer("test_operation");
        
        assert!(tracker.get_operation_time("test_operation").is_some());
        
        tracker.clear();
        
        assert!(tracker.get_operation_time("test_operation").is_none());
        assert_eq!(tracker.get_total_processing_time(), StdDuration::ZERO);
    }
    
    #[test]
    fn test_get_all_metrics() {
        let mut tracker = MetricsCollector::new();
        
        tracker.start_timer("op1");
        tracker.end_timer("op1");
        tracker.start_timer("op2");
        tracker.end_timer("op2");
        
        let all_metrics = tracker.get_all_metrics();
        assert_eq!(all_metrics.len(), 2);
        assert!(all_metrics.contains_key("op1"));
        assert!(all_metrics.contains_key("op2"));
    }
    
    #[test]
    fn test_noop_tracker() {
        let mut tracker = NoOpPerformanceTracker;
        
        // Should not panic
        tracker.start_timer("test");
        tracker.end_timer("test");
        tracker.report_metrics();
    }
    
    #[test]
    fn test_report_metrics_output() {
        let mut tracker = MetricsCollector::new();
        
        tracker.start_timer("fast_operation");
        tracker.end_timer("fast_operation");
        
        // Should not panic
        tracker.report_metrics();
    }
    
    #[test]
    fn test_report_metrics_disabled() {
        let tracker = MetricsCollector::new().with_enabled(false);
        
        // Should not panic and should not output anything
        tracker.report_metrics();
    }
}
