use crate::types::Primitive;

#[derive(Clone, Copy)]
pub enum Integer {
    I8(i8),
    U8(u8),
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
}

macro_rules! integer_from_integer {
    ($type:ident, $case:ident) => {
        impl From<$type> for Integer {
            #[inline]
            fn from(value: $type) -> Integer {
                Integer::$case(value)
            }
        }
    };
}

integer_from_integer!(i8, I8);
integer_from_integer!(u8, U8);
integer_from_integer!(i16, I16);
integer_from_integer!(u16, U16);
integer_from_integer!(i32, I32);
integer_from_integer!(u32, U32);
integer_from_integer!(i64, I64);
integer_from_integer!(u64, U64);

impl Integer {
    #[inline]
    pub fn primitive_type(&self) -> Primitive {
        match self {
            Integer::I8(_) => Primitive::I8,
            Integer::U8(_) => Primitive::U8,
            Integer::I16(_) => Primitive::I16,
            Integer::U16(_) => Primitive::U16,
            Integer::I32(_) => Primitive::I32,
            Integer::U32(_) => Primitive::U32,
            Integer::I64(_) => Primitive::I64,
            Integer::U64(_) => Primitive::U64,
        }
    }

    pub fn try_convert_to(&self, target: Primitive) -> Result<Integer, ()> {
        match self {
            Integer::I8(v) => match target {
                Primitive::I8 => Ok(*self),
                Primitive::U8 => (*v).try_into().map(|v| Self::U8(v)).map_err(|_| ()),
                Primitive::I16 => Ok(Self::I16(*v as i16)),
                Primitive::U16 => Ok(Self::U16(*v as u16)),
                Primitive::I32 => Ok(Self::I32(*v as i32)),
                Primitive::U32 => Ok(Self::U32(*v as u32)),
                Primitive::I64 => Ok(Self::I64(*v as i64)),
                Primitive::U64 => Ok(Self::U64(*v as u64)),
                _ => Err(()),
            },
            Integer::U8(v) => match target {
                Primitive::I8 => (*v).try_into().map(|v| Self::I8(v)).map_err(|_| ()),
                Primitive::U8 => Ok(*self),
                Primitive::I16 => Ok(Self::I16(*v as i16)),
                Primitive::U16 => Ok(Self::U16(*v as u16)),
                Primitive::I32 => Ok(Self::I32(*v as i32)),
                Primitive::U32 => Ok(Self::U32(*v as u32)),
                Primitive::I64 => Ok(Self::I64(*v as i64)),
                Primitive::U64 => Ok(Self::U64(*v as u64)),
                _ => Err(()),
            },
            Integer::I16(v) => match target {
                Primitive::I8 => (*v).try_into().map(|v| Self::I8(v)).map_err(|_| ()),
                Primitive::U8 => (*v).try_into().map(|v| Self::U8(v)).map_err(|_| ()),
                Primitive::I16 => Ok(*self),
                Primitive::U16 => (*v).try_into().map(|v| Self::U16(v)).map_err(|_| ()),
                Primitive::I32 => Ok(Self::I32(*v as i32)),
                Primitive::U32 => Ok(Self::U32(*v as u32)),
                Primitive::I64 => Ok(Self::I64(*v as i64)),
                Primitive::U64 => Ok(Self::U64(*v as u64)),
                _ => Err(()),
            },
            Integer::U16(v) => match target {
                Primitive::I8 => (*v).try_into().map(|v| Self::I8(v)).map_err(|_| ()),
                Primitive::U8 => (*v).try_into().map(|v| Self::U8(v)).map_err(|_| ()),
                Primitive::I16 => (*v).try_into().map(|v| Self::I16(v)).map_err(|_| ()),
                Primitive::U16 => Ok(*self),
                Primitive::I32 => Ok(Self::I32(*v as i32)),
                Primitive::U32 => Ok(Self::U32(*v as u32)),
                Primitive::I64 => Ok(Self::I64(*v as i64)),
                Primitive::U64 => Ok(Self::U64(*v as u64)),
                _ => Err(()),
            },
            Integer::I32(v) => match target {
                Primitive::I8 => (*v).try_into().map(|v| Self::I8(v)).map_err(|_| ()),
                Primitive::U8 => (*v).try_into().map(|v| Self::U8(v)).map_err(|_| ()),
                Primitive::I16 => (*v).try_into().map(|v| Self::I16(v)).map_err(|_| ()),
                Primitive::U16 => (*v).try_into().map(|v| Self::U16(v)).map_err(|_| ()),
                Primitive::I32 => Ok(*self),
                Primitive::U32 => (*v).try_into().map(|v| Self::U32(v)).map_err(|_| ()),
                Primitive::I64 => Ok(Self::I64(*v as i64)),
                Primitive::U64 => Ok(Self::U64(*v as u64)),
                _ => Err(()),
            },
            Integer::U32(v) => match target {
                Primitive::I8 => (*v).try_into().map(|v| Self::I8(v)).map_err(|_| ()),
                Primitive::U8 => (*v).try_into().map(|v| Self::U8(v)).map_err(|_| ()),
                Primitive::I16 => (*v).try_into().map(|v| Self::I16(v)).map_err(|_| ()),
                Primitive::U16 => (*v).try_into().map(|v| Self::U16(v)).map_err(|_| ()),
                Primitive::I32 => (*v).try_into().map(|v| Self::I32(v)).map_err(|_| ()),
                Primitive::U32 => Ok(*self),
                Primitive::I64 => Ok(Self::I64(*v as i64)),
                Primitive::U64 => Ok(Self::U64(*v as u64)),
                _ => Err(()),
            },
            Integer::I64(v) => match target {
                Primitive::I8 => (*v).try_into().map(|v| Self::I8(v)).map_err(|_| ()),
                Primitive::U8 => (*v).try_into().map(|v| Self::U8(v)).map_err(|_| ()),
                Primitive::I16 => (*v).try_into().map(|v| Self::I16(v)).map_err(|_| ()),
                Primitive::U16 => (*v).try_into().map(|v| Self::U16(v)).map_err(|_| ()),
                Primitive::I32 => (*v).try_into().map(|v| Self::I32(v)).map_err(|_| ()),
                Primitive::U32 => (*v).try_into().map(|v| Self::U32(v)).map_err(|_| ()),
                Primitive::I64 => Ok(*self),
                Primitive::U64 => (*v).try_into().map(|v| Self::U64(v)).map_err(|_| ()),
                _ => Err(()),
            },
            Integer::U64(v) => match target {
                Primitive::I8 => (*v).try_into().map(|v| Self::I8(v)).map_err(|_| ()),
                Primitive::U8 => (*v).try_into().map(|v| Self::U8(v)).map_err(|_| ()),
                Primitive::I16 => (*v).try_into().map(|v| Self::I16(v)).map_err(|_| ()),
                Primitive::U16 => (*v).try_into().map(|v| Self::U16(v)).map_err(|_| ()),
                Primitive::I32 => (*v).try_into().map(|v| Self::I32(v)).map_err(|_| ()),
                Primitive::U32 => (*v).try_into().map(|v| Self::U32(v)).map_err(|_| ()),
                Primitive::I64 => (*v).try_into().map(|v| Self::I64(v)).map_err(|_| ()),
                Primitive::U64 => Ok(*self),
                _ => Err(()),
            },
        }
    }

    pub fn singned_value(&self) -> Result<i64, ()> {
        self.try_convert_to(Primitive::I64).and_then(|v| match v {
            Self::I64(v) => Ok(v),
            _ => Err(()),
        })
    }

    pub fn unsingned_value(&self) -> Result<u64, ()> {
        self.try_convert_to(Primitive::U64).and_then(|v| match v {
            Self::U64(v) => Ok(v),
            _ => Err(()),
        })
    }

    #[inline]
    pub fn is_signed_integer(&self) -> bool {
        matches!(
            self,
            Self::I8(_) | Self::I16(_) | Self::I32(_) | Self::I64(_)
        )
    }

    #[inline]
    pub fn is_unsigned_integer(&self) -> bool {
        matches!(
            self,
            Self::U8(_) | Self::U16(_) | Self::U32(_) | Self::U64(_)
        )
    }

    #[inline]
    pub fn int_to_signed(&self, reverse_sign: bool) -> Result<Integer, Primitive> {
        match self {
            Integer::I8(_) | Integer::I16(_) | Integer::I32(_) | Integer::I64(_) => Ok(*self),
            Integer::U8(_) => self
                .try_convert_to(Primitive::I8)
                .map_err(|_| Primitive::I8),
            Integer::U16(_) => self
                .try_convert_to(Primitive::I16)
                .map_err(|_| Primitive::I16),
            Integer::U32(_) => self
                .try_convert_to(Primitive::I32)
                .map_err(|_| Primitive::I32),
            Integer::U64(_) => self
                .try_convert_to(Primitive::I64)
                .map_err(|_| Primitive::I64),
        }
        .map(|v| {
            if reverse_sign {
                match v {
                    Integer::I8(v) => Integer::I8(v.wrapping_neg()),
                    Integer::I16(v) => Integer::I16(v.wrapping_neg()),
                    Integer::I32(v) => Integer::I32(v.wrapping_neg()),
                    Integer::I64(v) => Integer::I64(v.wrapping_neg()),
                    _ => unreachable!(),
                }
            } else {
                v
            }
        })
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        match self {
            Integer::I8(v) => *v == 0,
            Integer::U8(v) => *v == 0,
            Integer::I16(v) => *v == 0,
            Integer::U16(v) => *v == 0,
            Integer::I32(v) => *v == 0,
            Integer::U32(v) => *v == 0,
            Integer::I64(v) => *v == 0,
            Integer::U64(v) => *v == 0,
        }
    }

    pub fn from_str(str: &str) -> Result<Self, ()> {
        match str.parse() {
            Ok(v) => Ok(Self::U64(v)),
            Err(_) => Err(()),
        }
    }
}

impl core::fmt::Debug for Integer {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::I8(arg0) => arg0.fmt(f),
            Self::U8(arg0) => arg0.fmt(f),
            Self::I16(arg0) => arg0.fmt(f),
            Self::U16(arg0) => arg0.fmt(f),
            Self::I32(arg0) => arg0.fmt(f),
            Self::U32(arg0) => arg0.fmt(f),
            Self::I64(arg0) => arg0.fmt(f),
            Self::U64(arg0) => arg0.fmt(f),
        }
    }
}
