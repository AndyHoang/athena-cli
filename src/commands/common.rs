use std::time::Duration;
use byte_unit::Byte;
use aws_sdk_athena::primitives::DateTime;

// Trait for converting values to display strings with a default fallback
pub trait DisplayValue {
    fn to_display_value(&self) -> String;
}

// Trait for converting Option<T> to display strings
pub trait OptionDisplayValue<T> {
    fn to_display_value_or_default(&self) -> String;
}

// Implementation for Option<T> where T implements DisplayValue
impl<T: DisplayValue> OptionDisplayValue<T> for Option<T> {
    fn to_display_value_or_default(&self) -> String {
        self.as_ref()
            .map(|v| v.to_display_value())
            .unwrap_or_else(|| "-".to_string())
    }
}

// Common implementations for basic types
impl DisplayValue for String {
    fn to_display_value(&self) -> String {
        self.clone()
    }
}

impl DisplayValue for &str {
    fn to_display_value(&self) -> String {
        self.to_string()
    }
}

impl DisplayValue for i64 {
    fn to_display_value(&self) -> String {
        self.to_string()
    }
}

// DateTime formatting
impl DisplayValue for DateTime {
    fn to_display_value(&self) -> String {
        self.to_string()
    }
}

impl DisplayValue for &DateTime {
    fn to_display_value(&self) -> String {
        (*self).to_string()
    }
}

// Duration formatting
impl DisplayValue for Duration {
    fn to_display_value(&self) -> String {
        humantime::format_duration(*self).to_string()
    }
}

// Bytes formatting
pub trait ByteDisplay {
    fn format_bytes(&self) -> String;
}

impl ByteDisplay for i64 {
    fn format_bytes(&self) -> String {
        Byte::from_i64(*self)
            .map(|b| b.get_appropriate_unit(byte_unit::UnitType::Decimal).to_string())
            .unwrap_or_else(|| "-".to_string())
    }
}

pub trait OptionByteDisplay {
    fn format_bytes_or_default(&self) -> String;
}

impl<T: Into<i64> + Copy> OptionByteDisplay for Option<T> {
    fn format_bytes_or_default(&self) -> String {
        self.map(|b| b.into().format_bytes())
            .unwrap_or_else(|| "-".to_string())
    }
}

// Helper for converting milliseconds to duration string
pub trait DurationFormat {
    fn format_duration_ms(&self) -> String;
}

impl DurationFormat for i64 {
    fn format_duration_ms(&self) -> String {
        Duration::from_millis(*self as u64).to_display_value()
    }
}

// Helper for Option<T> where T can be converted to duration
pub trait OptionDurationFormat {
    fn format_duration_ms_or_default(&self) -> String;
}

impl<T: Into<i64> + Copy> OptionDurationFormat for Option<T> {
    fn format_duration_ms_or_default(&self) -> String {
        self.map(|ms| ms.into().format_duration_ms())
            .unwrap_or_else(|| "-".to_string())
    }
} 