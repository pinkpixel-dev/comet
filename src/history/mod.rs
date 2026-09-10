pub mod ring_buffer;

pub use ring_buffer::RingBuffer;

/// Default number of samples retained for charts (~60-120 seconds of telemetry).
pub const HISTORY_CAPACITY: usize = 120;

/// Rolling time-series history for major system metrics.
#[derive(Debug, Clone)]
pub struct MetricHistory {
    pub cpu_overall: RingBuffer<f64>,
    pub ram_usage_percent: RingBuffer<f64>,
    pub ram_used_bytes: RingBuffer<f64>,
    pub swap_usage_percent: RingBuffer<f64>,
    pub gpu_utilization: RingBuffer<f64>,
    pub gpu_vram_percent: RingBuffer<f64>,
    pub net_rx_rate: RingBuffer<f64>,
    pub net_tx_rate: RingBuffer<f64>,
    pub disk_read_rate: RingBuffer<f64>,
    pub disk_write_rate: RingBuffer<f64>,
}

impl Default for MetricHistory {
    fn default() -> Self {
        Self::new(HISTORY_CAPACITY)
    }
}

impl MetricHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            cpu_overall: RingBuffer::new(capacity),
            ram_usage_percent: RingBuffer::new(capacity),
            ram_used_bytes: RingBuffer::new(capacity),
            swap_usage_percent: RingBuffer::new(capacity),
            gpu_utilization: RingBuffer::new(capacity),
            gpu_vram_percent: RingBuffer::new(capacity),
            net_rx_rate: RingBuffer::new(capacity),
            net_tx_rate: RingBuffer::new(capacity),
            disk_read_rate: RingBuffer::new(capacity),
            disk_write_rate: RingBuffer::new(capacity),
        }
    }
}
