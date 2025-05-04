pub trait ForceCast<T> {
    fn force_cast(self) -> T;
}

impl<T> ForceCast<T> for T {
    fn force_cast(self) -> T {
        self
    }
}

impl ForceCast<f32> for f64 {
    fn force_cast(self) -> f32 {
        self as _
    }
}

impl ForceCast<f32> for i64 {
    fn force_cast(self) -> f32 {
        self as _
    }
}

impl ForceCast<f32> for u64 {
    fn force_cast(self) -> f32 {
        self as _
    }
}

impl ForceCast<f32> for i32 {
    fn force_cast(self) -> f32 {
        self as _
    }
}

impl ForceCast<f32> for u32 {
    fn force_cast(self) -> f32 {
        self as f32
    }
}

impl ForceCast<f64> for f32 {
    fn force_cast(self) -> f64 {
        self as _
    }
}

impl ForceCast<f64> for i64 {
    fn force_cast(self) -> f64 {
        self as _
    }
}

impl ForceCast<f64> for u64 {
    fn force_cast(self) -> f64 {
        self as _
    }
}

impl ForceCast<f64> for i32 {
    fn force_cast(self) -> f64 {
        self as _
    }
}

impl ForceCast<f64> for u32 {
    fn force_cast(self) -> f64 {
        self as _
    }
}

impl ForceCast<u32> for f64 {
    fn force_cast(self) -> u32 {
        self as _
    }
}

impl ForceCast<u32> for f32 {
    fn force_cast(self) -> u32 {
        self as _
    }
}

impl ForceCast<u64> for f64 {
    fn force_cast(self) -> u64 {
        self as _
    }
}

impl ForceCast<u64> for f32 {
    fn force_cast(self) -> u64 {
        self as _
    }
}
