use std::ops;

use serde::{Deserialize, Serialize};

use crate::Float;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Angle<T = Float>
where
    T: AngleInner,
{
    degrees: T,
}

impl<T: AngleInner + Copy> Angle<T> {
    pub fn degrees(degrees: T) -> Self {
        Self { degrees }
    }

    pub fn as_degrees(&self) -> T {
        self.degrees
    }

    pub fn radians(radians: T) -> Self {
        Self {
            degrees: radians.to_degrees(),
        }
    }

    pub fn as_radians(&self) -> T {
        self.degrees.to_radians()
    }

    pub fn clamp(mut self, min: Self, max: Self) -> Self {
        self.degrees = self.degrees.clamp(min.degrees, max.degrees);
        self
    }
}

pub trait AngleInner {
    fn to_degrees(self) -> Self;

    fn to_radians(self) -> Self;

    fn clamp(self, min: Self, max: Self) -> Self;
}

macro_rules! angle_inner_impl {
    ($t:ty) => {
        impl AngleInner for $t {
            fn to_degrees(self) -> Self {
                self.to_degrees()
            }

            fn to_radians(self) -> Self {
                self.to_radians()
            }

            fn clamp(self, min: Self, max: Self) -> Self {
                self.clamp(min, max)
            }
        }

        impl From<$t> for Angle<$t> {
            fn from(value: $t) -> Self {
                Self { degrees: value }
            }
        }

        impl AsRef<$t> for Angle<$t> {
            fn as_ref(&self) -> &$t {
                &self.degrees
            }
        }

        impl AsMut<$t> for Angle<$t> {
            fn as_mut(&mut self) -> &mut $t {
                &mut self.degrees
            }
        }
    };
}

angle_inner_impl!(f32);
angle_inner_impl!(f64);

impl<T> ops::Add for Angle<T>
where
    T: AngleInner + ops::Add<Output = T>,
{
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            degrees: self.degrees + rhs.degrees,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::FRAC_PI_2;

    use super::*;

    const DEGREES_0: f64 = 0_f64;
    const DEGREES_45: f64 = 45_f64;
    const DEGREES_90: f64 = 90_f64;
    const DEGREES_180: f64 = 180_f64;

    #[test]
    fn test_angle_to_radians() {
        let angle = Angle::degrees(DEGREES_90);
        assert_eq!(angle.as_radians(), FRAC_PI_2);
    }

    #[test]
    fn test_angle_to_degrees() {
        let angle = Angle::radians(FRAC_PI_2);
        assert_eq!(angle.as_degrees(), DEGREES_90);
    }

    #[test]
    fn test_angle_add() {
        let lhs = Angle::degrees(DEGREES_90);
        let rhs = Angle::degrees(DEGREES_90);
        let result = lhs + rhs;
        assert_eq!(result.as_degrees(), DEGREES_180);
    }

    #[test]
    fn test_angle_clamp_max() {
        let angle = Angle::degrees(DEGREES_90);
        let min = Angle::degrees(DEGREES_0);
        let max = Angle::degrees(DEGREES_45);
        let result = angle.clamp(min, max);
        assert_eq!(result.as_degrees(), DEGREES_45);
    }

    #[test]
    fn test_angle_clamp_min() {
        let angle = Angle::degrees(-DEGREES_90);
        let min = Angle::degrees(DEGREES_0);
        let max = Angle::degrees(DEGREES_45);
        let result = angle.clamp(min, max);
        assert_eq!(result.as_degrees(), DEGREES_0);
    }
}
