pub use nalgebra;

pub use self::angle::Angle;
pub use self::pin::NodePin;
pub use self::ray::Ray;
pub use self::vector::{Basis, Vector, Vector2, Vector3, Vector4};

pub mod angle;
pub mod cast;
pub mod pin;
pub mod ray;
pub mod rect;
pub mod vector;

pub type Float = f64;

pub type Color = ecolor::Color32;
pub type Point2<T = Float> = Vector2<T>;
pub type Point3<T = Float> = Vector3<T>;

pub type Matrix2<T = Float> = nalgebra::Matrix2<T>;
pub type Matrix3<T = Float> = nalgebra::Matrix3<T>;
pub type Matrix4<T = Float> = nalgebra::Matrix4<T>;
