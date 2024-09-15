use toyir::Primitive;

#[derive(Clone, Copy)]
pub enum Float {
    F32(f32),
    F64(f64),
}

impl Float {
    #[inline]
    pub fn primitive_type(&self) -> Primitive {
        match self {
            Self::F32(_) => Primitive::F32,
            Self::F64(_) => Primitive::F64,
        }
    }

    pub fn try_convert_to(&self, target: Primitive) -> Result<Self, ()> {
        match self {
            Self::F32(v) => match target {
                Primitive::F32 => Ok(*self),
                Primitive::F64 => Ok(Self::F64(*v as f64)),
                _ => Err(()),
            },
            Self::F64(v) => match target {
                Primitive::F32 => {
                    let v = *v as f32;
                    v.is_finite().then(|| Self::F32(v)).ok_or(())
                }
                Primitive::F64 => Ok(*self),
                _ => Err(()),
            },
        }
    }
}

impl core::fmt::Debug for Float {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::F32(arg0) => arg0.fmt(f),
            Self::F64(arg0) => arg0.fmt(f),
        }
    }
}
