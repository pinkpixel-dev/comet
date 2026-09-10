use sysinfo::Components;

#[derive(Debug, Clone)]
pub struct SensorItem {
    pub label: String,
    pub temperature: f32,
    pub max_temp: Option<f32>,
    pub critical_temp: Option<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct SensorMetrics {
    pub items: Vec<SensorItem>,
    pub max_temperature: Option<f32>,
}

pub struct SensorSampler {
    components: Components,
}

impl SensorSampler {
    pub fn new() -> Self {
        let components = Components::new_with_refreshed_list();
        Self { components }
    }

    pub fn sample(&mut self) -> SensorMetrics {
        self.components.refresh(true);
        let mut items = Vec::new();
        let mut max_temperature: Option<f32> = None;

        for comp in self.components.iter() {
            if let Some(temp) = comp.temperature() {
                let label = comp.label().to_string();

                if let Some(cur_max) = max_temperature {
                    if temp > cur_max {
                        max_temperature = Some(temp);
                    }
                } else {
                    max_temperature = Some(temp);
                }

                items.push(SensorItem {
                    label,
                    temperature: temp,
                    max_temp: comp.max(),
                    critical_temp: comp.critical(),
                });
            }
        }

        // Sort sensors by temperature descending
        items.sort_by(|a, b| b.temperature.partial_cmp(&a.temperature).unwrap_or(std::cmp::Ordering::Equal));

        SensorMetrics {
            items,
            max_temperature,
        }
    }
}
