use std::ops::{Index, IndexMut};

use nalgebra::Scalar;
use serde::{Deserialize, Serialize};

use crate::Float;

pub type Vector2<T = Float> = nalgebra::Vector2<T>;
pub type Vector3<T = Float> = nalgebra::Vector3<T>;
pub type Vector4<T = Float> = nalgebra::Vector4<T>;

pub fn convert_vector3_down(v: &Vector3<f64>) -> Vector3<f32> {
    Vector3::new(v.x as _, v.y as _, v.z as _)
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Basis<T = Float>
where
    T: Scalar,
{
    pub u: Vector3<T>,
    pub v: Vector3<T>,
    pub w: Vector3<T>,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Vector {
    Dim2(Vector2),
    Dim3(Vector3),
    Dim4(Vector4),
}

impl Vector {
    pub fn from_scalar(scalar: Float) -> Self {
        Self::Dim4(Vector4::new(scalar, scalar, scalar, scalar))
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Dim2(_) => 2,
            Self::Dim3(_) => 3,
            Self::Dim4(_) => 4,
        }
    }

    pub fn as_dim2(&self) -> Vector2 {
        match self {
            Self::Dim2(v) => *v,
            Self::Dim3(v) => Vector2::new(v.x, v.y),
            Self::Dim4(v) => Vector2::new(v.x, v.y),
        }
    }

    pub fn as_dim3(&self) -> Vector3 {
        match self {
            Self::Dim2(v) => Vector3::new(v.x, v.y, 0.),
            Self::Dim3(v) => *v,
            Self::Dim4(v) => Vector3::new(v.x, v.y, v.z),
        }
    }

    pub fn as_dim4(&self) -> Vector4 {
        match self {
            Self::Dim2(v) => Vector4::new(v.x, v.y, 0., 0.),
            Self::Dim3(v) => Vector4::new(v.x, v.y, v.z, 0.),
            Self::Dim4(v) => *v,
        }
    }
}

impl Index<usize> for Vector {
    type Output = Float;

    #[inline]
    fn index(&self, i: usize) -> &Self::Output {
        match self {
            Self::Dim2(vector) => &vector[i],
            Self::Dim3(vector) => &vector[i],
            Self::Dim4(vector) => &vector[i],
        }
    }
}

impl IndexMut<usize> for Vector {
    #[inline]
    fn index_mut(&mut self, i: usize) -> &mut Float {
        match self {
            Self::Dim2(vector) => &mut vector[i],
            Self::Dim3(vector) => &mut vector[i],
            Self::Dim4(vector) => &mut vector[i],
        }
    }
}
