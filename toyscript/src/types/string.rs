use crate::*;

#[derive(Debug)]
pub struct StringTable {
    data: Vec<u8>,
    descriptors: Vec<StringDescriptor>,
}

impl StringTable {
    #[inline]
    pub fn new() -> Self {
        Self {
            data: Default::default(),
            descriptors: Default::default(),
        }
    }

    pub fn register(&mut self, s: &str) -> StringIndex {
        match self.find(s) {
            Some(v) => return v,
            None => {}
        }

        let index = StringIndex(self.descriptors.len() as u32);
        let desc = StringDescriptor {
            offset: self.data.len() as u32,
            len: s.len() as u32,
        };
        self.data.extend_from_slice(s.as_bytes());
        self.descriptors.push(desc);

        index
    }

    pub fn find(&self, s: &str) -> Option<StringIndex> {
        for (index, _) in self.descriptors.iter().enumerate() {
            let index = StringIndex(index as u32);
            if self.get_string(index) == s {
                return Some(index);
            }
        }
        None
    }

    pub fn get_descriptor(&self, index: StringIndex) -> StringDescriptor {
        unsafe {
            let desc = self.descriptors.get_unchecked(index.0 as usize);
            *desc
        }
    }

    pub fn get_string(&self, index: StringIndex) -> &str {
        let desc = self.get_descriptor(index);
        unsafe {
            core::str::from_utf8_unchecked(
                self.data
                    .get_unchecked(desc.offset as usize..(desc.offset + desc.len) as usize),
            )
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct StringDescriptor {
    offset: u32,
    len: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StringIndex(u32);

impl StringIndex {
    #[inline]
    pub const fn as_u32(&self) -> u32 {
        self.0
    }

    #[inline]
    pub const fn as_usize(&self) -> usize {
        self.0 as usize
    }
}
