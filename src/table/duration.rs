use crate::{PklResult, PklValue};
use std::fmt;
use std::{ops::Range, time::Duration as StdDuration};

// pub const DURATION_UNITS: [&str; 7] = ["ns", "us", "ms", "s", "min", "h", "d"];

/// Based on v0.26.0
pub fn match_duration_props_api<'a>(
    duration: Duration<'a>,
    property: &'a str,
    range: Range<usize>,
) -> PklResult<PklValue<'a>> {
    match property {
        "value" => {
            return Ok(*duration.initial_value);
        }
        "unit" => {
            return Ok(PklValue::String(duration.unit.to_string()));
        }
        "isPositive" => return Ok(PklValue::Bool(!duration.is_negative)),
        _ => {
            return Err((
                format!("DataSize does not possess {} property", property),
                range,
            ))
        }
    }
}

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

#[derive(Debug, Clone, PartialEq)]
pub struct Duration<'a> {
    duration: StdDuration,
    initial_value: Box<PklValue<'a>>,
    unit: Unit,
    is_negative: bool,
}

impl<'a> Duration<'a> {
    pub fn from_float_and_unit(value: f64, unit: Unit) -> Self {
        let initial_value = Box::new(PklValue::Float(value));
        let is_negative = value.is_sign_negative();
        let value = if is_negative { value.abs() } else { value };

        let duration = match unit {
            Unit::NS => StdDuration::from_secs_f64(value * 10e-9),
            Unit::US => StdDuration::from_secs_f64(value * 10e-6),
            Unit::MS => StdDuration::from_secs_f64(value * 10e-3),
            Unit::S => StdDuration::from_secs_f64(value),
            Unit::MIN => StdDuration::from_secs_f64(value * 60.0),
            Unit::H => StdDuration::from_secs_f64(value * 60.0 * 60.0),
            Unit::D => StdDuration::from_secs_f64(value * 60.0 * 60.0 * 24.0),
        };

        Self {
            duration,
            unit,
            initial_value,
            is_negative,
        }
    }

    pub fn from_int_and_unit(value: i64, unit: Unit) -> Self {
        let initial_value = Box::new(PklValue::Int(value));
        let is_negative = value < 0;
        let value = if is_negative {
            (value as f64).abs()
        } else {
            value as f64
        };

        let duration = match unit {
            Unit::NS => StdDuration::from_secs_f64(value * 10e-9),
            Unit::US => StdDuration::from_secs_f64(value * 10e-6),
            Unit::MS => StdDuration::from_secs_f64(value * 10e-3),
            Unit::S => StdDuration::from_secs_f64(value),
            Unit::MIN => StdDuration::from_secs_f64(value * 60.0),
            Unit::H => StdDuration::from_secs_f64(value * 60.0 * 60.0),
            Unit::D => StdDuration::from_secs_f64(value * 60.0 * 60.0 * 24.0),
        };

        Self {
            duration,
            unit,
            initial_value,
            is_negative,
        }
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unit_str = match self {
            Unit::NS => "ns",
            Unit::US => "us",
            Unit::MS => "ms",
            Unit::S => "s",
            Unit::MIN => "min",
            Unit::H => "h",
            Unit::D => "d",
        };
        write!(f, "{}", unit_str)
    }
}
