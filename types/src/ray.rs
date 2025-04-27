use std::ops;

use nalgebra::{Scalar, SimdComplexField};

use crate::{Float, Point3, Vector3};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Ray<T = Float>
where
    T: Scalar,
{
    pub origin: Point3<T>,
    pub direction: Vector3<T>,
}

impl<T: Scalar + SimdComplexField> Ray<T> {
    pub fn new(origin: Point3<T>, direction: Vector3<T>) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }
}

impl<T> Ray<T>
where
    T: Scalar + Copy + ops::Mul<Vector3<T>, Output = Vector3<T>> + ops::Add<T, Output = T> + ops::AddAssign<T>,
{
    pub fn at(&self, t: T) -> Point3<T> {
        self.origin + t * self.direction
    }
}
