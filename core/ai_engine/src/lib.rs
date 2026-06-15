//! # AI Engine
//!
//! Local AI subsystem for PUCE providing:
//! - Analog stick auto-calibration
//! - Drift detection and real-time compensation
//! - Latency optimization
//! - Intelligent profile suggestion
//! - Auto-mapping heuristics
//! - Kalman filter for gyroscope stabilization

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use thiserror::Error;

// ─────────────────────────────────────────────────────────────
// Error types
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum AIError {
    #[error("insufficient samples for calibration: need {needed}, have {have}")]
    InsufficientSamples { needed: usize, have: usize },
    #[error("calibration data invalid: {0}")]
    InvalidCalibration(String),
}

// ─────────────────────────────────────────────────────────────
// Vector types
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self { Self { x, y } }

    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn distance_to(&self, other: &Vec2) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn normalized(&self) -> Self {
        let mag = self.magnitude();
        if mag < f32::EPSILON {
            return *self;
        }
        Self::new(self.x / mag, self.y / mag)
    }
}

// ─────────────────────────────────────────────────────────────
// Calibration Data
// ─────────────────────────────────────────────────────────────

/// Calibration state for a single analog stick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StickCalibration {
    /// The resting center point (normalized -1.0..1.0)
    pub center: Vec2,
    /// Dead zone radius — inputs within this radius snap to 0
    pub dead_zone: f32,
    /// Effective maximum radius (usually < 1.0 due to physical limits)
    pub max_radius: f32,
    /// Asymmetric max per quadrant [top, right, bottom, left]
    pub quadrant_max: [f32; 4],
}

impl Default for StickCalibration {
    fn default() -> Self {
        Self {
            center: Vec2::default(),
            dead_zone: 0.08, // 8% dead zone
            max_radius: 1.0,
            quadrant_max: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

/// Full device calibration (both sticks + triggers).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceCalibration {
    pub device_id: String,
    pub left_stick: StickCalibration,
    pub right_stick: StickCalibration,
    pub l2_min: u8,
    pub l2_max: u8,
    pub r2_min: u8,
    pub r2_max: u8,
}

// ─────────────────────────────────────────────────────────────
// Auto-Calibrator
// ─────────────────────────────────────────────────────────────

/// Calibrates analog sticks from a set of raw samples.
pub struct AutoCalibrator {
    /// Samples collected during calibration (normalized -1..1)
    center_samples: Vec<Vec2>,
    range_samples: Vec<Vec2>,
    min_center_samples: usize,
    min_range_samples: usize,
}

impl AutoCalibrator {
    pub fn new() -> Self {
        Self {
            center_samples: Vec::new(),
            range_samples: Vec::new(),
            min_center_samples: 60,  // ~1 second at 60fps
            min_range_samples: 120,
        }
    }

    /// Add a sample during center calibration phase (stick at rest).
    pub fn add_center_sample(&mut self, x: f32, y: f32) {
        self.center_samples.push(Vec2::new(x, y));
    }

    /// Add a sample during range calibration phase (stick moved to all extremes).
    pub fn add_range_sample(&mut self, x: f32, y: f32) {
        self.range_samples.push(Vec2::new(x, y));
    }

    /// Compute calibration from collected samples.
    pub fn compute(&self) -> Result<StickCalibration, AIError> {
        if self.center_samples.len() < self.min_center_samples {
            return Err(AIError::InsufficientSamples {
                needed: self.min_center_samples,
                have: self.center_samples.len(),
            });
        }

        // Compute center as mean of resting samples
        let sum_x: f32 = self.center_samples.iter().map(|s| s.x).sum();
        let sum_y: f32 = self.center_samples.iter().map(|s| s.y).sum();
        let count = self.center_samples.len() as f32;
        let center = Vec2::new(sum_x / count, sum_y / count);

        // Compute dead zone from center sample spread (2-sigma)
        let var_x: f32 = self.center_samples.iter()
            .map(|s| (s.x - center.x).powi(2))
            .sum::<f32>() / count;
        let var_y: f32 = self.center_samples.iter()
            .map(|s| (s.y - center.y).powi(2))
            .sum::<f32>() / count;
        let spread = (var_x.sqrt() + var_y.sqrt()) / 2.0;
        let dead_zone = (spread * 3.0).max(0.04).min(0.20); // 4%–20%

        // Compute max radius from range samples
        let max_radius = if !self.range_samples.is_empty() {
            self.range_samples.iter()
                .map(|s| {
                    let dx = s.x - center.x;
                    let dy = s.y - center.y;
                    (dx * dx + dy * dy).sqrt()
                })
                .fold(f32::NEG_INFINITY, f32::max)
                .min(1.0)
        } else {
            1.0
        };

        // Compute per-quadrant maximums
        let quadrant_max = self.compute_quadrant_max(&center);

        Ok(StickCalibration {
            center,
            dead_zone,
            max_radius: max_radius.max(dead_zone + 0.1),
            quadrant_max,
        })
    }

    fn compute_quadrant_max(&self, center: &Vec2) -> [f32; 4] {
        let mut max = [0.01f32; 4]; // top, right, bottom, left
        for s in &self.range_samples {
            let dx = s.x - center.x;
            let dy = s.y - center.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dy < 0.0 { max[0] = max[0].max(dist); } // top
            if dx > 0.0 { max[1] = max[1].max(dist); } // right
            if dy > 0.0 { max[2] = max[2].max(dist); } // bottom
            if dx < 0.0 { max[3] = max[3].max(dist); } // left
        }
        for m in &mut max {
            if *m < 0.1 { *m = 1.0; } // fallback if not enough range data
        }
        max
    }

    pub fn reset(&mut self) {
        self.center_samples.clear();
        self.range_samples.clear();
    }
}

impl Default for AutoCalibrator {
    fn default() -> Self { Self::new() }
}

// ─────────────────────────────────────────────────────────────
// Input Normalizer — applies calibration in real-time
// ─────────────────────────────────────────────────────────────

/// Applies calibration data to raw stick values in real-time.
pub struct InputNormalizer;

impl InputNormalizer {
    /// Normalize a raw stick value (0–255 u8) to calibrated -1.0..1.0.
    /// Applies: centering → dead zone → range scaling → curve.
    pub fn normalize_axis(raw: u8, cal: &StickCalibration) -> f32 {
        // Map 0–255 to -1.0..1.0
        let normalized = (raw as f32 / 127.5) - 1.0;

        // Subtract center offset
        let x = normalized - cal.center.x;

        // Apply dead zone (radial snap to 0)
        if x.abs() < cal.dead_zone {
            return 0.0;
        }

        // Scale to full range accounting for dead zone
        let effective_range = cal.max_radius - cal.dead_zone;
        if effective_range <= 0.0 {
            return 0.0;
        }

        let sign = if x >= 0.0 { 1.0 } else { -1.0 };
        let abs_val = (x.abs() - cal.dead_zone) / effective_range;
        (sign * abs_val.min(1.0))
    }

    /// Normalize a pair of raw stick axes (x, y) with radial dead zone.
    pub fn normalize_stick(raw_x: u8, raw_y: u8, cal: &StickCalibration) -> (f32, f32) {
        let nx = (raw_x as f32 / 127.5) - 1.0 - cal.center.x;
        let ny = (raw_y as f32 / 127.5) - 1.0 - cal.center.y;
        let mag = (nx * nx + ny * ny).sqrt();

        if mag < cal.dead_zone {
            return (0.0, 0.0);
        }

        let effective_range = cal.max_radius - cal.dead_zone;
        if effective_range <= 0.0 {
            return (0.0, 0.0);
        }

        let scale = ((mag - cal.dead_zone) / effective_range).min(1.0) / mag;
        (nx * scale, ny * scale)
    }

    /// Map normalized -1.0..1.0 back to u8 (0–255) for HID report.
    pub fn to_u8(normalized: f32) -> u8 {
        let clamped = normalized.max(-1.0).min(1.0);
        ((clamped + 1.0) * 127.5).round() as u8
    }
}

// ─────────────────────────────────────────────────────────────
// Drift Detector
// ─────────────────────────────────────────────────────────────

/// Detects analog stick drift using a sliding window baseline.
pub struct DriftDetector {
    /// Ring buffer of recent samples when stick appears to be at rest
    baseline_buffer: VecDeque<Vec2>,
    /// Running average of the baseline
    baseline: Vec2,
    /// Current correction offset
    pub correction: Vec2,
    /// Drift threshold — if baseline deviates more than this, drift is flagged
    drift_threshold: f32,
    /// Whether drift has been detected
    pub drift_detected: bool,
    /// Sample capacity
    capacity: usize,
}

impl DriftDetector {
    pub fn new(drift_threshold: f32) -> Self {
        Self {
            baseline_buffer: VecDeque::with_capacity(300),
            baseline: Vec2::default(),
            correction: Vec2::default(),
            drift_threshold,
            drift_detected: false,
            capacity: 300,
        }
    }

    /// Update with a new stick reading (normalized -1..1 each axis).
    /// Returns the corrected value.
    pub fn update(&mut self, x: f32, y: f32) -> Vec2 {
        let input = Vec2::new(x, y);

        // Only update baseline if stick appears to be near center (magnitude < 0.15)
        if input.magnitude() < 0.15 {
            if self.baseline_buffer.len() >= self.capacity {
                self.baseline_buffer.pop_front();
            }
            self.baseline_buffer.push_back(input);

            if self.baseline_buffer.len() >= 60 {
                self.update_baseline();
            }
        }

        // Apply correction
        Vec2::new(
            (x - self.correction.x).max(-1.0).min(1.0),
            (y - self.correction.y).max(-1.0).min(1.0),
        )
    }

    fn update_baseline(&mut self) {
        let n = self.baseline_buffer.len() as f32;
        let sum_x: f32 = self.baseline_buffer.iter().map(|v| v.x).sum();
        let sum_y: f32 = self.baseline_buffer.iter().map(|v| v.y).sum();
        let new_baseline = Vec2::new(sum_x / n, sum_y / n);

        // Check if baseline has drifted significantly from (0,0)
        if new_baseline.magnitude() > self.drift_threshold {
            self.drift_detected = true;
            self.correction = new_baseline;
            log::warn!(
                "Stick drift detected! Offset: ({:.3}, {:.3}), applying correction.",
                new_baseline.x, new_baseline.y
            );
        } else {
            self.drift_detected = false;
        }

        self.baseline = new_baseline;
    }

    /// Reset drift correction (e.g., after recalibration).
    pub fn reset(&mut self) {
        self.baseline_buffer.clear();
        self.baseline = Vec2::default();
        self.correction = Vec2::default();
        self.drift_detected = false;
    }
}

// ─────────────────────────────────────────────────────────────
// Kalman Filter (for gyroscope stabilization)
// ─────────────────────────────────────────────────────────────

/// Simple 1D Kalman filter for sensor noise reduction.
pub struct KalmanFilter1D {
    /// Estimated state
    x: f32,
    /// Estimate uncertainty
    p: f32,
    /// Process noise variance (Q)
    q: f32,
    /// Measurement noise variance (R)
    r: f32,
}

impl KalmanFilter1D {
    pub fn new(q: f32, r: f32) -> Self {
        Self { x: 0.0, p: 1.0, q, r }
    }

    pub fn gyro_filter() -> Self {
        Self::new(0.001, 0.1) // Low process noise, moderate measurement noise
    }

    pub fn accel_filter() -> Self {
        Self::new(0.003, 0.5)
    }

    /// Process a new measurement and return the filtered estimate.
    pub fn update(&mut self, measurement: f32) -> f32 {
        // Prediction step
        self.p += self.q;

        // Update step
        let k = self.p / (self.p + self.r); // Kalman gain
        self.x += k * (measurement - self.x);
        self.p *= 1.0 - k;

        self.x
    }

    pub fn reset(&mut self) {
        self.x = 0.0;
        self.p = 1.0;
    }
}

/// Applies Kalman filtering to 3-axis gyroscope data.
pub struct GyroStabilizer {
    filters: [KalmanFilter1D; 3],
}

impl GyroStabilizer {
    pub fn new() -> Self {
        Self {
            filters: [
                KalmanFilter1D::gyro_filter(),
                KalmanFilter1D::gyro_filter(),
                KalmanFilter1D::gyro_filter(),
            ],
        }
    }

    pub fn filter(&mut self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        (
            self.filters[0].update(x),
            self.filters[1].update(y),
            self.filters[2].update(z),
        )
    }
}

impl Default for GyroStabilizer {
    fn default() -> Self { Self::new() }
}

// ─────────────────────────────────────────────────────────────
// Latency Optimizer
// ─────────────────────────────────────────────────────────────

/// Connection type for latency tuning.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionType {
    USB,
    Bluetooth,
    BLE,
}

/// Recommends optimal polling rates and buffering strategies.
pub struct LatencyOptimizer;

impl LatencyOptimizer {
    /// Recommended polling rate in Hz for a given connection type.
    pub fn recommended_polling_rate(conn: ConnectionType) -> u32 {
        match conn {
            ConnectionType::USB => 1000,
            ConnectionType::Bluetooth => 250,
            ConnectionType::BLE => 125,
        }
    }

    /// Recommended input buffer depth (frames) for given polling rate.
    pub fn recommended_buffer_depth(polling_hz: u32) -> u32 {
        match polling_hz {
            1000 => 1,
            500 => 1,
            250 => 2,
            _ => 3,
        }
    }

    /// Jitter smoothing alpha for exponential moving average (0..1).
    pub fn ema_alpha(conn: ConnectionType) -> f32 {
        match conn {
            ConnectionType::USB => 0.95,       // Minimal smoothing
            ConnectionType::Bluetooth => 0.80,
            ConnectionType::BLE => 0.70,
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Exponential Moving Average filter
// ─────────────────────────────────────────────────────────────

/// Smooths analog axis values to reduce jitter.
pub struct EMAFilter {
    alpha: f32,
    value: f32,
    initialized: bool,
}

impl EMAFilter {
    pub fn new(alpha: f32) -> Self {
        Self { alpha, value: 0.0, initialized: false }
    }

    pub fn update(&mut self, new_value: f32) -> f32 {
        if !self.initialized {
            self.value = new_value;
            self.initialized = true;
        } else {
            self.value = self.alpha * new_value + (1.0 - self.alpha) * self.value;
        }
        self.value
    }

    pub fn reset(&mut self) {
        self.initialized = false;
        self.value = 0.0;
    }
}

// ─────────────────────────────────────────────────────────────
// Profile Suggester
// ─────────────────────────────────────────────────────────────

/// Game genre classification for profile optimization.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GameType {
    FPS,
    RPG,
    PlatformerAction,
    RacingDriving,
    FightingGame,
    SportsGame,
    StrategyRTS,
    Unknown,
}

/// Device capability summary for suggestion purposes.
#[derive(Debug, Clone)]
pub struct DeviceSummary {
    pub button_count: u8,
    pub axis_count: u8,
    pub has_gyro: bool,
    pub has_touchpad: bool,
    pub has_vibration: bool,
    pub vendor_id: u16,
    pub is_keyboard: bool,
    pub is_mouse: bool,
}

/// Suggests mapping profiles based on device type and game context.
pub struct ProfileSuggester;

impl ProfileSuggester {
    /// Suggest the optimal PlayStation mode for a given device.
    pub fn suggest_ps_mode(device: &DeviceSummary) -> crate::PSModeHint {
        if device.is_keyboard || device.is_mouse {
            return crate::PSModeHint::DualShock4; // DS4 for mouse touchpad simulation
        }

        // Sony first-party → pick highest capability
        if device.vendor_id == 0x054C {
            if device.has_gyro && device.has_touchpad && device.has_vibration {
                return crate::PSModeHint::DualSense;
            }
            if device.has_touchpad {
                return crate::PSModeHint::DualShock4;
            }
            if device.has_gyro {
                return crate::PSModeHint::Sixaxis;
            }
        }

        // Xbox → DS4 is best match (same button count layout)
        if device.vendor_id == 0x045E || device.vendor_id == 0x0E6F {
            return crate::PSModeHint::DualShock4;
        }

        // Nintendo Switch Pro → DS4
        if device.vendor_id == 0x057E {
            return crate::PSModeHint::DualShock4;
        }

        // Generic gamepad with sticks → DS4
        if device.axis_count >= 4 && device.button_count >= 10 {
            return crate::PSModeHint::DualShock4;
        }

        // Simple 2-axis, 6-button gamepad → DualShock
        if device.axis_count >= 2 {
            return crate::PSModeHint::DualShock1;
        }

        crate::PSModeHint::DualShock4
    }

    /// Suggest dead zone adjustments per game type.
    pub fn suggest_dead_zone(game_type: GameType) -> f32 {
        match game_type {
            GameType::FPS => 0.05,           // Very responsive
            GameType::RacingDriving => 0.02, // Ultra precise steering
            GameType::FightingGame => 0.08,  // Standard
            GameType::RPG => 0.10,           // Larger, less precise movement
            GameType::PlatformerAction => 0.06,
            GameType::SportsGame => 0.07,
            _ => 0.08,
        }
    }

    /// Suggest response curve per game type.
    pub fn suggest_curve(game_type: GameType) -> ResponseCurve {
        match game_type {
            GameType::FPS => ResponseCurve::SCurve,     // Fine control near center
            GameType::RacingDriving => ResponseCurve::Linear,
            GameType::FightingGame => ResponseCurve::Digital, // Snap to cardinal
            _ => ResponseCurve::EaseIn,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ResponseCurve {
    Linear,
    EaseIn,
    EaseOut,
    SCurve,
    Digital, // Snap to ±1 once past dead zone
}

impl ResponseCurve {
    /// Apply the response curve to a normalized stick value (-1..1).
    pub fn apply(&self, value: f32) -> f32 {
        match self {
            ResponseCurve::Linear => value,
            ResponseCurve::EaseIn => value.signum() * value.abs().powi(2),
            ResponseCurve::EaseOut => value.signum() * (1.0 - (1.0 - value.abs()).powi(2)),
            ResponseCurve::SCurve => {
                // Cubic S-curve: smoother center, strong edges
                value.signum() * (3.0 * value.abs().powi(2) - 2.0 * value.abs().powi(3))
            }
            ResponseCurve::Digital => {
                if value.abs() > 0.5 { value.signum() } else { 0.0 }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────
// PS Mode Hint (re-exported from emulation to avoid circular dep)
// ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PSModeHint {
    PS1, DualAnalog, DualShock1, DualShock2,
    Sixaxis, DualShock3, DualShock4, DualSense, DualSenseEdge,
}

// ─────────────────────────────────────────────────────────────
// Main AI Engine
// ─────────────────────────────────────────────────────────────

/// Orchestrates all AI subsystems. One instance per connected device.
pub struct AIEngine {
    pub calibrator: AutoCalibrator,
    pub left_drift: DriftDetector,
    pub right_drift: DriftDetector,
    pub gyro_stabilizer: GyroStabilizer,
    pub left_x_filter: EMAFilter,
    pub left_y_filter: EMAFilter,
    pub right_x_filter: EMAFilter,
    pub right_y_filter: EMAFilter,
    pub calibration: DeviceCalibration,
    pub auto_correct_drift: bool,
}

impl AIEngine {
    pub fn new(device_id: String) -> Self {
        let cal = DeviceCalibration {
            device_id,
            ..Default::default()
        };
        Self {
            calibrator: AutoCalibrator::new(),
            left_drift: DriftDetector::new(0.03),
            right_drift: DriftDetector::new(0.03),
            gyro_stabilizer: GyroStabilizer::new(),
            left_x_filter: EMAFilter::new(0.90),
            left_y_filter: EMAFilter::new(0.90),
            right_x_filter: EMAFilter::new(0.90),
            right_y_filter: EMAFilter::new(0.90),
            calibration: cal,
            auto_correct_drift: true,
        }
    }

    /// Process raw stick input (0–255 each), returning corrected normalized values.
    pub fn process_left_stick(&mut self, raw_x: u8, raw_y: u8) -> (f32, f32) {
        let (nx, ny) = InputNormalizer::normalize_stick(
            raw_x, raw_y, &self.calibration.left_stick,
        );
        let fx = self.left_x_filter.update(nx);
        let fy = self.left_y_filter.update(ny);
        if self.auto_correct_drift {
            let corrected = self.left_drift.update(fx, fy);
            (corrected.x, corrected.y)
        } else {
            (fx, fy)
        }
    }

    pub fn process_right_stick(&mut self, raw_x: u8, raw_y: u8) -> (f32, f32) {
        let (nx, ny) = InputNormalizer::normalize_stick(
            raw_x, raw_y, &self.calibration.right_stick,
        );
        let fx = self.right_x_filter.update(nx);
        let fy = self.right_y_filter.update(ny);
        if self.auto_correct_drift {
            let corrected = self.right_drift.update(fx, fy);
            (corrected.x, corrected.y)
        } else {
            (fx, fy)
        }
    }

    /// Process gyroscope data through Kalman filter.
    pub fn process_gyro(&mut self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        self.gyro_stabilizer.filter(x, y, z)
    }

    /// Apply calibration from completed calibrator.
    pub fn apply_calibration(&mut self) -> Result<(), AIError> {
        let left_cal = self.calibrator.compute()?;
        self.calibration.left_stick = left_cal;
        self.left_drift.reset();
        self.right_drift.reset();
        log::info!("Calibration applied for device: {}", self.calibration.device_id);
        Ok(())
    }

    /// Check if drift is currently being corrected.
    pub fn drift_status(&self) -> DriftStatus {
        DriftStatus {
            left_drifting: self.left_drift.drift_detected,
            right_drifting: self.right_drift.drift_detected,
            left_correction: self.left_drift.correction,
            right_correction: self.right_drift.correction,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftStatus {
    pub left_drifting: bool,
    pub right_drifting: bool,
    pub left_correction: Vec2,
    pub right_correction: Vec2,
}

// ─────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kalman_filter_converges() {
        let mut kf = KalmanFilter1D::gyro_filter();
        // Feed constant noisy signal
        for _ in 0..100 {
            kf.update(1.0 + (fastrand_f32() - 0.5) * 0.1);
        }
        let estimate = kf.update(1.0);
        assert!((estimate - 1.0).abs() < 0.05, "Kalman didn't converge: {}", estimate);
    }

    #[test]
    fn test_drift_detector_detects_drift() {
        let mut dd = DriftDetector::new(0.02);
        // Feed samples simulating drift toward (0.1, 0.0)
        for _ in 0..300 {
            dd.update(0.05, 0.0);
        }
        assert!(dd.drift_detected, "Drift should have been detected");
        assert!(dd.correction.x.abs() > 0.01);
    }

    #[test]
    fn test_drift_detector_no_false_positive() {
        let mut dd = DriftDetector::new(0.03);
        // Feed centered samples with tiny noise
        for i in 0..300 {
            let noise = if i % 2 == 0 { 0.005 } else { -0.005 };
            dd.update(noise, noise);
        }
        assert!(!dd.drift_detected, "Should not detect drift for tiny noise");
    }

    #[test]
    fn test_ema_filter_smoothing() {
        let mut ema = EMAFilter::new(0.5);
        ema.update(1.0);
        let v = ema.update(0.0);
        // After one step with alpha=0.5: 0.5*0 + 0.5*1.0 = 0.5
        assert!((v - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_response_curve_scurve() {
        let curve = ResponseCurve::SCurve;
        assert_eq!(curve.apply(0.0), 0.0);
        assert!((curve.apply(1.0) - 1.0).abs() < 0.001);
        assert!((curve.apply(-1.0) + 1.0).abs() < 0.001);
    }

    #[test]
    fn test_input_normalizer_center() {
        let cal = StickCalibration::default();
        // Raw 128 should normalize close to 0 (within dead zone)
        let v = InputNormalizer::normalize_axis(128, &cal);
        assert_eq!(v, 0.0, "Center should be zero");
    }

    #[test]
    fn test_input_normalizer_max() {
        let cal = StickCalibration { dead_zone: 0.0, ..Default::default() };
        let v = InputNormalizer::normalize_axis(255, &cal);
        assert!((v - 1.0).abs() < 0.01, "Max raw should be near 1.0");
    }

    fn fastrand_f32() -> f32 {
        // Deterministic pseudo-random for tests
        0.5_f32
    }
}
