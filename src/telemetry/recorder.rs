use crate::telemetry::TelemetryState;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingFormat {
    Csv,
    Json,
}

impl RecordingFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            RecordingFormat::Csv => "csv",
            RecordingFormat::Json => "json",
        }
    }
}

/// Structured telemetry sample ready for export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingSample {
    pub timestamp_utc: String,
    pub elapsed_seconds: f64,
    pub cpu_overall_percent: f32,
    pub cpu_cores_count: usize,
    pub ram_used_bytes: u64,
    pub ram_total_bytes: u64,
    pub ram_used_percent: f32,
    pub swap_used_bytes: u64,
    pub swap_total_bytes: u64,
    pub swap_used_percent: f32,
    pub gpu_vendor: String,
    pub gpu_name: String,
    pub gpu_utilization_percent: f32,
    pub gpu_vram_used_bytes: u64,
    pub gpu_vram_total_bytes: u64,
    pub gpu_temperature_c: Option<f32>,
    pub gpu_power_watts: Option<f32>,
    pub disk_read_bytes_sec: f64,
    pub disk_write_bytes_sec: f64,
    pub net_rx_bytes_sec: f64,
    pub net_tx_bytes_sec: f64,
    pub max_sensor_temp_c: Option<f32>,
}

struct ActiveRecording {
    path: PathBuf,
    format: RecordingFormat,
    start_time: Instant,
    sample_count: usize,
    writer: BufWriter<File>,
}

pub struct TelemetryRecorder {
    pub format: RecordingFormat,
    pub output_dir: PathBuf,
    active: Option<ActiveRecording>,
}

impl TelemetryRecorder {
    pub fn new(format: RecordingFormat, output_dir: PathBuf) -> Self {
        Self {
            format,
            output_dir,
            active: None,
        }
    }

    pub fn is_recording(&self) -> bool {
        self.active.is_some()
    }

    pub fn active_path(&self) -> Option<&Path> {
        self.active.as_ref().map(|a| a.path.as_path())
    }

    pub fn sample_count(&self) -> usize {
        self.active.as_ref().map(|a| a.sample_count).unwrap_or(0)
    }

    pub fn elapsed(&self) -> Duration {
        self.active
            .as_ref()
            .map(|a| a.start_time.elapsed())
            .unwrap_or(Duration::ZERO)
    }

    /// Starts a new recording session, creating the file and writing headers.
    pub fn start(&mut self) -> Result<PathBuf, std::io::Error> {
        if let Some(active) = &self.active {
            return Ok(active.path.clone());
        }

        fs::create_dir_all(&self.output_dir)?;

        let (_, file_ts) = format_utc_timestamp(SystemTime::now());
        let filename = format!("comet_telemetry_{}.{}", file_ts, self.format.extension());
        let filepath = self.output_dir.join(filename);

        let file = File::create(&filepath)?;
        let mut writer = BufWriter::new(file);

        match self.format {
            RecordingFormat::Csv => {
                writeln!(
                    writer,
                    "timestamp_utc,elapsed_seconds,cpu_overall_percent,cpu_cores_count,ram_used_bytes,ram_total_bytes,ram_used_percent,swap_used_bytes,swap_total_bytes,swap_used_percent,gpu_vendor,gpu_name,gpu_util_percent,gpu_vram_used_bytes,gpu_vram_total_bytes,gpu_temp_c,gpu_power_watts,disk_read_bytes_sec,disk_write_bytes_sec,net_rx_bytes_sec,net_tx_bytes_sec,max_sensor_temp_c"
                )?;
                writer.flush()?;
            }
            RecordingFormat::Json => {
                writeln!(writer, "[")?;
                writer.flush()?;
            }
        }

        self.active = Some(ActiveRecording {
            path: filepath.clone(),
            format: self.format,
            start_time: Instant::now(),
            sample_count: 0,
            writer,
        });

        Ok(filepath)
    }

    /// Records a single telemetry snapshot into the active file.
    pub fn record(
        &mut self,
        state: &TelemetryState,
        selected_gpu_index: usize,
    ) -> Result<(), std::io::Error> {
        let active = match &mut self.active {
            Some(a) => a,
            None => return Ok(()),
        };

        let elapsed = active.start_time.elapsed().as_secs_f64();
        let (iso_ts, _) = format_utc_timestamp(SystemTime::now());

        let active_gpu = state
            .gpus
            .get(selected_gpu_index)
            .or(state.gpu.as_ref());

        let sample = RecordingSample {
            timestamp_utc: iso_ts,
            elapsed_seconds: (elapsed * 1000.0).round() / 1000.0,
            cpu_overall_percent: state.cpu.overall_usage,
            cpu_cores_count: state.cpu.core_usages.len(),
            ram_used_bytes: state.memory.used_bytes,
            ram_total_bytes: state.memory.total_bytes,
            ram_used_percent: state.memory.usage_percent(),
            swap_used_bytes: state.memory.swap_used_bytes,
            swap_total_bytes: state.memory.swap_total_bytes,
            swap_used_percent: state.memory.swap_usage_percent(),
            gpu_vendor: active_gpu.map(|g| g.vendor.to_string()).unwrap_or_else(|| "none".to_string()),
            gpu_name: active_gpu.map(|g| g.name.clone()).unwrap_or_else(|| "none".to_string()),
            gpu_utilization_percent: active_gpu.map(|g| g.utilization).unwrap_or(0.0),
            gpu_vram_used_bytes: active_gpu.map(|g| g.memory_used).unwrap_or(0),
            gpu_vram_total_bytes: active_gpu.map(|g| g.memory_total).unwrap_or(0),
            gpu_temperature_c: active_gpu.and_then(|g| g.temperature),
            gpu_power_watts: active_gpu.and_then(|g| g.power_usage_watts),
            disk_read_bytes_sec: state.total_disk_read_rate,
            disk_write_bytes_sec: state.total_disk_write_rate,
            net_rx_bytes_sec: state.network.total_rx_rate,
            net_tx_bytes_sec: state.network.total_tx_rate,
            max_sensor_temp_c: state.sensors.max_temperature,
        };

        match active.format {
            RecordingFormat::Csv => {
                let escaped_gpu_name = format!("\"{}\"", sample.gpu_name.replace('"', "\"\""));
                writeln!(
                    active.writer,
                    "{},{:.3},{:.2},{},{},{},{:.2},{},{},{:.2},{},{},{:.2},{},{},{},{},{:.2},{:.2},{:.2},{:.2},{}",
                    sample.timestamp_utc,
                    sample.elapsed_seconds,
                    sample.cpu_overall_percent,
                    sample.cpu_cores_count,
                    sample.ram_used_bytes,
                    sample.ram_total_bytes,
                    sample.ram_used_percent,
                    sample.swap_used_bytes,
                    sample.swap_total_bytes,
                    sample.swap_used_percent,
                    sample.gpu_vendor,
                    escaped_gpu_name,
                    sample.gpu_utilization_percent,
                    sample.gpu_vram_used_bytes,
                    sample.gpu_vram_total_bytes,
                    sample.gpu_temperature_c.map(|v| format!("{:.1}", v)).unwrap_or_default(),
                    sample.gpu_power_watts.map(|v| format!("{:.1}", v)).unwrap_or_default(),
                    sample.disk_read_bytes_sec,
                    sample.disk_write_bytes_sec,
                    sample.net_rx_bytes_sec,
                    sample.net_tx_bytes_sec,
                    sample.max_sensor_temp_c.map(|v| format!("{:.1}", v)).unwrap_or_default()
                )?;
                active.writer.flush()?;
            }
            RecordingFormat::Json => {
                let json_str = serde_json::to_string(&sample)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                if active.sample_count > 0 {
                    writeln!(active.writer, ",")?;
                }
                write!(active.writer, "  {}", json_str)?;
                active.writer.flush()?;
            }
        }

        active.sample_count += 1;
        Ok(())
    }

    /// Stops the recording session, finalizing the file and returning summary stats.
    pub fn stop(&mut self) -> Result<Option<(PathBuf, usize, Duration)>, std::io::Error> {
        let mut active = match self.active.take() {
            Some(a) => a,
            None => return Ok(None),
        };

        if active.format == RecordingFormat::Json {
            writeln!(active.writer)?;
            writeln!(active.writer, "]")?;
        }

        active.writer.flush()?;

        let elapsed = active.start_time.elapsed();
        Ok(Some((active.path, active.sample_count, elapsed)))
    }
}

impl Drop for TelemetryRecorder {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// Converts a `SystemTime` into UTC ISO 8601 string and safe filename timestamp.
/// Howard Hinnant civil day algorithm for Gregorian calendar computation without external deps.
pub fn format_utc_timestamp(time: SystemTime) -> (String, String) {
    let dur = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let total_secs = dur.as_secs();
    let total_days = total_secs / 86400;
    let sec_of_day = total_secs % 86400;

    let hour = sec_of_day / 3600;
    let minute = (sec_of_day % 3600) / 60;
    let second = sec_of_day % 60;

    let z = total_days as i64 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1029 + doe / 1461 - doe / 36524) / 365;
    let y = (yoe as i64 + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let y = if m <= 2 { y + 1 } else { y };

    let iso_ts = format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, hour, minute, second);
    let file_ts = format!("{:04}{:02}{:02}_{:02}{:02}{:02}", y, m, d, hour, minute, second);
    (iso_ts, file_ts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_utc_timestamp_epoch() {
        let (iso, file) = format_utc_timestamp(UNIX_EPOCH);
        assert_eq!(iso, "1970-01-01T00:00:00Z");
        assert_eq!(file, "19700101_000000");
    }

    #[test]
    fn test_utc_timestamp_known_date() {
        // 2026-09-10 00:00:00 UTC = 20706 days * 86400 = 1788998400 seconds
        let known = UNIX_EPOCH + Duration::from_secs(1788998400);
        let (iso, file) = format_utc_timestamp(known);
        assert_eq!(iso, "2026-09-10T00:00:00Z");
        assert_eq!(file, "20260910_000000");
    }

    #[test]
    fn test_recorder_csv_lifecycle() {
        let temp_dir = std::env::temp_dir().join("comet_test_csv_record");
        let mut recorder = TelemetryRecorder::new(RecordingFormat::Csv, temp_dir.clone());

        assert!(!recorder.is_recording());
        let path = recorder.start().expect("Failed to start recorder");
        assert!(recorder.is_recording());
        assert_eq!(recorder.sample_count(), 0);

        let state = TelemetryState::default();
        recorder.record(&state, 0).expect("Failed to record sample");
        assert_eq!(recorder.sample_count(), 1);

        let result = recorder.stop().expect("Failed to stop recorder");
        assert!(!recorder.is_recording());
        assert!(result.is_some());

        let (saved_path, count, _) = result.unwrap();
        assert_eq!(saved_path, path);
        assert_eq!(count, 1);

        let content = fs::read_to_string(&saved_path).expect("Failed to read recorded CSV");
        assert!(content.contains("timestamp_utc,elapsed_seconds"));
        assert!(content.lines().count() >= 2); // Header + 1 sample

        let _ = fs::remove_file(saved_path);
        let _ = fs::remove_dir(temp_dir);
    }

    #[test]
    fn test_recorder_json_lifecycle() {
        let temp_dir = std::env::temp_dir().join("comet_test_json_record");
        let mut recorder = TelemetryRecorder::new(RecordingFormat::Json, temp_dir.clone());

        let _path = recorder.start().expect("Failed to start recorder");
        let state = TelemetryState::default();
        recorder.record(&state, 0).expect("Failed to record sample 1");
        recorder.record(&state, 0).expect("Failed to record sample 2");

        let result = recorder.stop().expect("Failed to stop recorder");
        let (saved_path, count, _) = result.unwrap();
        assert_eq!(count, 2);

        let content = fs::read_to_string(&saved_path).expect("Failed to read recorded JSON");
        // Verify it is valid JSON
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&content).expect("Valid JSON array");
        assert_eq!(parsed.len(), 2);

        let _ = fs::remove_file(saved_path);
        let _ = fs::remove_dir(temp_dir);
    }
}
