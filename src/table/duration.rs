use std::time::Duration;

// pub const DURATION_UNITS: [&str; 7] = ["ns", "us", "ms", "s", "min", "h", "d"];

/// An enum representing duration units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    NS,
    US,
    MS,
    S,
    MIN,
    H,
    D,
}

impl Unit {
    /// Parses a string slice into an `Option<Unit>`.
    /// Returns `None` if the string does not correspond to a known data size unit.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ns" => Some(Unit::NS),
            "us" => Some(Unit::US),
            "ms" => Some(Unit::MS),
            "s" => Some(Unit::S),
            "min" => Some(Unit::MIN),
            "h" => Some(Unit::H),
            "d" => Some(Unit::D),
            _ => None,
        }
    }
}

pub fn duration_from_value_and_unit(value: f64, unit: Unit) -> Duration {
    match unit {
        Unit::NS => Duration::from_secs_f64(value * 10e-9),
        Unit::US => Duration::from_secs_f64(value * 10e-6),
        Unit::MS => Duration::from_secs_f64(value * 10e-3),
        Unit::S => Duration::from_secs_f64(value),
        Unit::MIN => Duration::from_secs_f64(value * 60.0),
        Unit::H => Duration::from_secs_f64(value * 60.0 * 60.0),
        Unit::D => Duration::from_secs_f64(value * 60.0 * 60.0 * 24.0),
    }
}
