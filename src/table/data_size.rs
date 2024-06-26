// pub const DATA_SIZE_UNITS: [&str; 11] = [
//     "b", "kb", "mb", "gb", "tb", "pb", "kib", "mib", "gib", "tib", "pib",
// ];

/// An enum representing both binary (kibibytes, mebibytes, etc.)
/// and decimal (kilobytes, megabytes, etc.) data size units.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    B,
    KB,
    MB,
    GB,
    TB,
    PB,
    KiB,
    MiB,
    GiB,
    TiB,
    PiB,
}

impl Unit {
    /// Parses a string slice into an `Option<Unit>`.
    /// Returns `None` if the string does not correspond to a known data size unit.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "b" => Some(Unit::B),
            "kb" => Some(Unit::KB),
            "mb" => Some(Unit::MB),
            "gb" => Some(Unit::GB),
            "tb" => Some(Unit::TB),
            "pb" => Some(Unit::PB),
            "kib" => Some(Unit::KiB),
            "mib" => Some(Unit::MiB),
            "gib" => Some(Unit::GiB),
            "tib" => Some(Unit::TiB),
            "pib" => Some(Unit::PiB),
            _ => None,
        }
    }
}

/// Represents data sizes in bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Byte {
    bytes: u64,
}

impl Byte {
    /// Creates a new `Byte` from a floating point value and a unit.
    ///
    /// # Arguments
    /// * `value` - The numeric value of the data size.
    /// * `unit` - The unit of the data size (`Unit`).
    ///
    /// # Returns
    /// Returns a new `Byte` representing the size in bytes.
    pub fn from_value_and_unit(value: f64, unit: Unit) -> Self {
        let bytes = match unit {
            Unit::B => value,
            Unit::KB => value * 1_000.0,
            Unit::MB => value * 1_000_000.0,
            Unit::GB => value * 1_000_000_000.0,
            Unit::TB => value * 1_000_000_000_000.0,
            Unit::PB => value * 1_000_000_000_000_000.0,
            Unit::KiB => value * 1_024.0,
            Unit::MiB => value * 1_024.0 * 1_024.0,
            Unit::GiB => value * 1_024.0 * 1_024.0 * 1_024.0,
            Unit::TiB => value * 1_024.0 * 1_024.0 * 1_024.0 * 1_024.0,
            Unit::PiB => value * 1_024.0 * 1_024.0 * 1_024.0 * 1_024.0 * 1_024.0,
        };

        Byte {
            bytes: bytes as u64,
        }
    }
}
