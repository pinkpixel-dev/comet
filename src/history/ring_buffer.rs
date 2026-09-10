use std::collections::VecDeque;

/// A fixed-capacity ring buffer for time-series metrics.
#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    data: VecDeque<T>,
    capacity: usize,
}

impl<T: Clone> RingBuffer<T> {
    /// Creates a new ring buffer with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
            capacity: capacity.max(1),
        }
    }

    /// Appends a new item, evicting the oldest if capacity is reached.
    pub fn push(&mut self, item: T) {
        if self.data.len() >= self.capacity {
            self.data.pop_front();
        }
        self.data.push_back(item);
    }

    /// Returns the number of items in the buffer.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the buffer has no items.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the most recently pushed item, if any.
    pub fn latest(&self) -> Option<&T> {
        self.data.back()
    }

    /// Returns an iterator over the items in chronological order (oldest first).
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    /// Converts the buffer items to a flat Vec in chronological order.
    pub fn to_vec(&self) -> Vec<T> {
        self.data.iter().cloned().collect()
    }
}

impl RingBuffer<f64> {
    /// Helper to convert samples into `(x, y)` coordinate tuples for Ratatui charts.
    pub fn as_chart_data(&self) -> Vec<(f64, f64)> {
        self.data
            .iter()
            .enumerate()
            .map(|(i, &val)| (i as f64, val))
            .collect()
    }

    /// Helper to convert samples into `u64` values for Ratatui sparklines.
    pub fn as_sparkline_data(&self, max_scale: f64) -> Vec<u64> {
        self.data
            .iter()
            .map(|&val| {
                if max_scale <= 0.0 {
                    val.max(0.0) as u64
                } else {
                    ((val / max_scale) * 100.0).clamp(0.0, 100.0) as u64
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_capacity() {
        let mut buf = RingBuffer::new(3);
        buf.push(1);
        buf.push(2);
        buf.push(3);
        assert_eq!(buf.to_vec(), vec![1, 2, 3]);
        buf.push(4);
        assert_eq!(buf.to_vec(), vec![2, 3, 4]);
        assert_eq!(buf.latest(), Some(&4));
    }
}
