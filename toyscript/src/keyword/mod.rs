//! ToyScript Reserved Keywords

mod _keyword;
pub use _keyword::*;
use token::Token;

impl Keyword {
    pub fn modifiers() -> &'static [Self] {
        &[
            Keyword::Async,
            Keyword::Export,
            Keyword::Get,
            Keyword::Private,
            Keyword::Protected,
            Keyword::Public,
            Keyword::Set,
            Keyword::Static,
        ]
    }

    pub fn type_identifiers() -> &'static [Self] {
        &[
            Keyword::Any,
            Keyword::Boolean,
            Keyword::Number,
            Keyword::String,
            Keyword::Void,
        ]
    }

    pub fn constant_values() -> &'static [Self] {
        &[
            Keyword::False,
            Keyword::Null,
            Keyword::Super,
            Keyword::This,
            Keyword::True,
        ]
    }

    #[inline]
    pub fn is_modifier(&self) -> bool {
        Self::modifiers().contains(self)
    }

    #[inline]
    pub fn is_type_identifier(&self) -> bool {
        Self::type_identifiers().contains(self)
    }

    #[inline]
    pub fn is_constant_value(&self) -> bool {
        Self::constant_values().contains(self)
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct ModifierFlag(u32);

impl ModifierFlag {
    #[rustfmt::skip]
    pub const ASYNC: Self       = Self(0b0000_0001);
    #[rustfmt::skip]
    pub const EXPORT: Self      = Self(0b0000_0100);
    #[rustfmt::skip]
    pub const PRIVATE: Self     = Self(0b0001_0000);
    #[rustfmt::skip]
    pub const PROTECTED: Self   = Self(0b0010_0000);
    #[rustfmt::skip]
    pub const PUBLIC: Self      = Self(0b0100_0000);
    #[rustfmt::skip]
    pub const STATIC: Self      = Self(0b1000_0000);

    const ALL_CAES: &'static [Self] = &[
        Self::ASYNC,
        Self::EXPORT,
        Self::PRIVATE,
        Self::PROTECTED,
        Self::PUBLIC,
        Self::STATIC,
    ];

    #[inline]
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Self> + 'a {
        Self::ALL_CAES
            .iter()
            .filter_map(|v| self.contains(*v).then(|| *v))
    }

    #[inline]
    fn debug_iter<'a>(&'a self) -> impl Iterator<Item = &'static str> + 'a {
        self.iter().map(move |v| {
            if v.contains(Self::ASYNC) {
                "async"
            } else if v.contains(Self::EXPORT) {
                "export"
            } else if v.contains(Self::PRIVATE) {
                "private"
            } else if v.contains(Self::PROTECTED) {
                "declare"
            } else if v.contains(Self::PUBLIC) {
                "public"
            } else if v.contains(Self::STATIC) {
                "static"
            } else {
                ""
            }
        })
    }

    pub fn combined<'a>(iter: impl Iterator<Item = &'a Self>) -> Self {
        let mut result = Self::empty();
        for item in iter {
            result.insert(*item);
        }
        result
    }

    pub fn from_tokens(tokens: &[Token<Keyword>], accept: &[Self]) -> Result<Self, Token<Keyword>> {
        let mut result = Self::empty();
        let accept = Self::combined(accept.iter());
        for token in tokens {
            let keyword = token.clone().try_into_keyword()?;
            let new_value = match keyword.keyword() {
                Keyword::Async => Self::ASYNC,
                Keyword::Export => Self::EXPORT,
                Keyword::Public => Self::PUBLIC,
                Keyword::Private => Self::PRIVATE,
                Keyword::Protected => Self::PROTECTED,
                Keyword::Static => Self::STATIC,
                _ => return Err(token.clone()),
            };
            if (new_value.0 & accept.0) != new_value.0 {
                return Err(token.clone());
            }
            result.insert(new_value);
        }

        Ok(result)
    }

    pub fn from_keywords(keywords: impl Iterator<Item = Keyword>) -> Result<Self, Keyword> {
        let mut result = Self::empty();
        for keyword in keywords {
            match keyword {
                Keyword::Async => result.insert(Self::ASYNC),
                Keyword::Export => result.insert(Self::EXPORT),
                Keyword::Public => result.insert(Self::PUBLIC),
                Keyword::Private => result.insert(Self::PRIVATE),
                Keyword::Protected => result.insert(Self::PROTECTED),
                Keyword::Static => result.insert(Self::STATIC),
                _ => return Err(keyword),
            }
        }
        Ok(result)
    }

    #[inline]
    pub const fn empty() -> Self {
        Self(0)
    }

    #[inline]
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0;
    }

    #[inline]
    pub fn remove(&mut self, other: Self) {
        self.0 &= !other.0;
    }

    #[inline]
    pub fn set(&mut self, flag: Self, value: bool) {
        if value {
            self.insert(flag);
        } else {
            self.remove(flag);
        }
    }

    #[inline]
    pub const fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl core::fmt::Debug for ModifierFlag {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_set().entries(self.debug_iter()).finish()
    }
}
