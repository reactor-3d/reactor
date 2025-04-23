pub use self::pin::NodePin;

pub mod angle;
pub mod pin;
pub mod ray;

pub type Vector3 = nalgebra::Vector3<f64>;
pub type Point3 = Vector3;
pub type Matrix3 = nalgebra::Matrix3<f64>;
pub type Matrix4 = nalgebra::Matrix4<f64>;
pub type Matrix4f32 = nalgebra::Matrix4<f32>;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Basis {
    pub u: Vector3,
    pub v: Vector3,
    pub w: Vector3,
}
