//! The module for customized parsing of types for the command-line program.
//!
//! The two objects, `CustomDateObject` and `DecimalWrapper` are simple wrappers over existing
//! data types -- `chrono::NaiveDateTime` in the case of `CustomDateObject` and `rust_decimal::Decimal`
//! in the case of `DecimalWrapper`.
//!
//! This is necessary because `chrono` doesn't use `FromStr` (because it doesn't know the format it needs to parse)
//! and because I wanted to return the number of days between datetimes for range (overwriting `std::ops::Sub`).
//! And decimal has a way of parsing values in scientific notation and parsing normal numbers. So I added
//! the scientific notation parsing to the implementation of `FromStr`.
use chrono::NaiveDateTime;
use once_cell::sync::OnceCell;
use rust_decimal::Decimal;
use std::fmt;

const OUTPUT_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
/// The user-entered date format, used to allow `CustomDateObject` to run `FromStr` without needing an input string.
pub static INPUT_DATE_FORMAT: OnceCell<&str> = OnceCell::new();

/// Sets `INPUT_DATE_FORMAT`.
// this sets static variable, so there's no need to do anything here
#[allow(unused_must_use)]
pub fn set_date_format(s: &'static str) {
    INPUT_DATE_FORMAT.set(s);
}

/// A light wrapper over `rust_decimal::Decimal`.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct DecimalWrapper {
    pub item: Decimal,
}

impl std::str::FromStr for DecimalWrapper {
    type Err = rust_decimal::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Decimal::from_str(s)
            .or_else(|_| Decimal::from_scientific(s))
            .map(|v| DecimalWrapper { item: v })
    }
}

impl fmt::Display for DecimalWrapper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.item.to_string())
    }
}

// necessary to get range to work
impl std::ops::Sub for DecimalWrapper {
    type Output = Decimal;

    /// Returns the total number of days between two dates
    fn sub(self, other: DecimalWrapper) -> Decimal {
        self.item - other.item
    }
}

// necessary for Sum
impl std::ops::AddAssign for DecimalWrapper {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            item: self.item + other.item,
        }
    }
}

impl std::ops::Add for DecimalWrapper {
    type Output = DecimalWrapper;

    fn add(self, other: Self) -> Self {
        Self {
            item: self.item + other.item,
        }
    }
}

impl std::ops::Div for DecimalWrapper {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        DecimalWrapper {
            item: self.item / rhs.item,
        }
    }
}

/// A light wrapper over `chrono::NaiveDateTime`. Also implements `std::ops::Sub` to compute the total number of
/// days between two dates. This is probably not smart, but it allows me to easily run `Range` on dates.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct CustomDateObject(NaiveDateTime);

impl std::str::FromStr for CustomDateObject {
    type Err = chrono::format::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parsed_dt = NaiveDateTime::parse_from_str(s, INPUT_DATE_FORMAT.get().unwrap())?;
        Ok(CustomDateObject(parsed_dt))
    }
}

// necessary to get range to work
impl std::ops::Sub for CustomDateObject {
    type Output = f64;

    /// Returns the total number of days between two dates
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, other: CustomDateObject) -> f64 {
        let duration = self.0.signed_duration_since(other.0);
        duration.num_seconds() as f64 / 86400.
    }
}

impl fmt::Display for CustomDateObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format(OUTPUT_DATE_FORMAT))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use proptest::prelude::*;

    #[test]
    fn test_date_subtraction() {
        let day_recent = CustomDateObject(NaiveDate::from_ymd(2019, 1, 1).and_hms(0, 0, 0));
        let day_previous = CustomDateObject(NaiveDate::from_ymd(2018, 12, 31).and_hms(0, 0, 0));
        assert_eq!(day_recent - day_previous, 1.);
    }

    #[test]
    fn test_scientific_notation() {
        let scinot1: DecimalWrapper = "1e-4".parse().unwrap();
        assert_eq!(scinot1.to_string(), "0.0001".to_string());
        let scinot2: DecimalWrapper = "1.3E4".parse().unwrap();
        assert_eq!(scinot2.to_string(), "13000".to_string());
    }

    proptest! {
        #[test]
        fn test_date_parsing(year in 1900..=2020i32, month in 1..=12u32, day in 1..=28u32, hour in 0..=23u32, minute in 0..=59u32, second in 0..=59u32) {
            let dt = CustomDateObject(NaiveDate::from_ymd(year, month, day).and_hms(hour, minute, second));
            let _ex = INPUT_DATE_FORMAT.get_or_init(|| "%Y-%m-%d %H:%M:%S");
            let deser_ser : CustomDateObject = dt.to_string().parse().unwrap();
            assert_eq!(dt, deser_ser);
        }
        #[test]
        fn test_parses_decimal_normal(num in -1000000..=1000000i64, scale in 0..=16u32) {
            let dec = Decimal::new(num, scale);
            let decimal_wrapper = DecimalWrapper { item: dec };
            let deser_ser : DecimalWrapper = decimal_wrapper.to_string().parse().unwrap();
            assert_eq!(deser_ser.item, dec);
        }
    }
}
