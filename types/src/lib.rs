use nalgebra::Scalar;

pub use self::angle::Angle;
pub use self::pin::NodePin;
pub use self::ray::Ray;

pub mod angle;
pub mod pin;
pub mod ray;

pub type Float = f64;
pub type Color = Vector3<Float>;
pub type Point3<T = Float> = Vector3<T>;
pub type Vector3<T = Float> = nalgebra::Vector3<T>;
pub type Vector4<T = Float> = nalgebra::Vector4<T>;
pub type Matrix3<T = Float> = nalgebra::Matrix3<T>;
pub type Matrix4<T = Float> = nalgebra::Matrix4<T>;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Basis<T = Float>
where
    T: Scalar,
{
    pub u: Vector3<T>,
    pub v: Vector3<T>,
    pub w: Vector3<T>,
}

pub fn convert_vector3_to_f32(v: &Vector3<f64>) -> Vector3<f32> {
    Vector3::new(v.x as _, v.y as _, v.z as _)
}
