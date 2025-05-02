use crate::Float;

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct RectSize<T = Float> {
    pub width: T,
    pub height: T,
}

impl<T: Clone> From<T> for RectSize<T> {
    fn from(value: T) -> Self {
        RectSize {
            width: value.clone(),
            height: value,
        }
    }
}

impl<T> From<(T, T)> for RectSize<T> {
    fn from((width, height): (T, T)) -> Self {
        RectSize { width, height }
    }
}

impl<T> From<RectSize<T>> for (T, T) {
    fn from(rect_size: RectSize<T>) -> Self {
        (rect_size.width, rect_size.height)
    }
}
