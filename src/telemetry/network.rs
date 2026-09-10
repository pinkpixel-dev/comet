use std::collections::HashMap;
use std::time::Instant;
use sysinfo::Networks;

#[derive(Debug, Clone)]
pub struct InterfaceMetrics {
    pub name: String,
    pub rx_bytes_per_sec: f64,
    pub tx_bytes_per_sec: f64,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    pub total_rx_rate: f64,
    pub total_tx_rate: f64,
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
    pub interfaces: Vec<InterfaceMetrics>,
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self {
            total_rx_rate: 0.0,
            total_tx_rate: 0.0,
            total_rx_bytes: 0,
            total_tx_bytes: 0,
            interfaces: Vec::new(),
        }
    }
}

pub struct NetworkSampler {
    networks: Networks,
    last_sample_time: Instant,
    last_totals: HashMap<String, (u64, u64)>, // (rx_total, tx_total)
}

impl NetworkSampler {
    pub fn new() -> Self {
        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh(true);
        let mut last_totals = HashMap::new();
        for (name, net) in &networks {
            last_totals.insert(name.clone(), (net.total_received(), net.total_transmitted()));
        }

        Self {
            networks,
            last_sample_time: Instant::now(),
            last_totals,
        }
    }

    pub fn sample(&mut self) -> NetworkMetrics {
        self.networks.refresh(true);
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_sample_time).as_secs_f64();
        self.last_sample_time = now;

        let mut total_rx_rate = 0.0;
        let mut total_tx_rate = 0.0;
        let mut sum_rx_bytes = 0;
        let mut sum_tx_bytes = 0;
        let mut interfaces = Vec::new();

        for (name, net) in &self.networks {
            let cur_rx_total = net.total_received();
            let cur_tx_total = net.total_transmitted();

            sum_rx_bytes += cur_rx_total;
            sum_tx_bytes += cur_tx_total;

            let (prev_rx, prev_tx) = self
                .last_totals
                .get(name)
                .copied()
                .unwrap_or((cur_rx_total, cur_tx_total));

            let rx_rate = if elapsed > 0.0 {
                (cur_rx_total.saturating_sub(prev_rx) as f64) / elapsed
            } else {
                0.0
            };

            let tx_rate = if elapsed > 0.0 {
                (cur_tx_total.saturating_sub(prev_tx) as f64) / elapsed
            } else {
                0.0
            };

            total_rx_rate += rx_rate;
            total_tx_rate += tx_rate;

            interfaces.push(InterfaceMetrics {
                name: name.clone(),
                rx_bytes_per_sec: rx_rate,
                tx_bytes_per_sec: tx_rate,
                total_rx_bytes: cur_rx_total,
                total_tx_bytes: cur_tx_total,
            });

            self.last_totals.insert(name.clone(), (cur_rx_total, cur_tx_total));
        }

        // Sort interfaces so active ones are first
        interfaces.sort_by(|a, b| {
            let a_active = a.rx_bytes_per_sec + a.tx_bytes_per_sec;
            let b_active = b.rx_bytes_per_sec + b.tx_bytes_per_sec;
            b_active.partial_cmp(&a_active).unwrap_or(std::cmp::Ordering::Equal)
        });

        NetworkMetrics {
            total_rx_rate,
            total_tx_rate,
            total_rx_bytes: sum_rx_bytes,
            total_tx_bytes: sum_tx_bytes,
            interfaces,
        }
    }
}
