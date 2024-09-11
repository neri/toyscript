use std::{
    collections::BTreeMap,
    fs::{read_to_string, File},
    io::*,
};

fn main() {
    {
        make_primitive(
            "./src/primitive/primitive.csv",
            "./src/primitive/_primitive.rs",
            "Primitive",
            "ToyScript Primitive Types",
        );
    }

    {
        make_irop(
            "./src/opcode/irop.csv",
            "./src/opcode/_irop.rs",
            "Op",
            "ToyScript Intermediate Representation Opcodes",
        );
    }
}

#[derive(Debug)]
struct PrimitiveTypeDesc {
    name: String,
    identifier: String,
    bits: usize,
    size_of: usize,
    align_of: usize,
    kind: TypeKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TypeKind {
    Integer,
    Unsigned,
    Float,
    Void,
}

impl TypeKind {
    fn from_str(str: &str) -> Option<Self> {
        match str {
            "i" => Some(Self::Integer),
            "u" => Some(Self::Unsigned),
            "f" => Some(Self::Float),
            "v" => Some(Self::Void),
            _ => None,
        }
    }
}

fn make_primitive(src_path: &str, dest_path: &str, class_name: &str, comment: &str) {
    let mut primitives = BTreeMap::<String, PrimitiveTypeDesc>::new();

    for line in read_to_string(src_path).unwrap().lines().skip(1) {
        let mut cols = line.split(",");
        let name = cols.next().unwrap().to_owned();
        let bits = cols.next().unwrap().parse::<usize>().unwrap();
        let size_of = cols.next().unwrap().parse::<usize>().unwrap();
        let align_of = cols.next().unwrap().parse::<usize>().unwrap();
        let kind = TypeKind::from_str(cols.next().unwrap()).unwrap();

        let primitive = PrimitiveTypeDesc {
            name: name.clone(),
            identifier: to_camel_case_identifier(&name),
            bits,
            size_of,
            align_of,
            kind,
        };

        for item in primitives.values() {
            if item.name == primitive.name {
                panic!("Redefined type {}", primitive.name)
            }
            if item.kind != TypeKind::Void
                && item.kind == primitive.kind
                && item.bits == primitive.bits
            {
                panic!("Redundant type {}:{}", item.name, primitive.name)
            }
        }

        primitives.insert(name, primitive);
    }

    let mut os = File::create(dest_path).unwrap();

    write!(
        os,
        "//! {comment}

/* This file is generated automatically. DO NOT EDIT DIRECTLY. */

/// {comment}
#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum {class_name} {{
"
    )
    .unwrap();
    for keyword in primitives.keys() {
        writeln!(os, "    /// \"{}\"", keyword).unwrap();
        writeln!(os, "    {},", to_camel_case_identifier(keyword)).unwrap();
    }
    write!(
        os,
        "}}

impl {class_name} {{
    pub fn all_values() -> &'static [Self] {{
        &[
"
    )
    .unwrap();

    for keyword in primitives.keys() {
        writeln!(
            os,
            "            Self::{},",
            to_camel_case_identifier(keyword),
        )
        .unwrap();
    }

    write!(
        os,
        "        ]
    }}

    pub fn from_str(v: &str) -> Option<Self> {{
        match v {{
"
    )
    .unwrap();

    for keyword in primitives.keys() {
        writeln!(
            os,
            "            {:?} => Some(Self::{}),",
            keyword,
            to_camel_case_identifier(keyword),
        )
        .unwrap();
    }

    write!(
        os,
        "            _ => None,
        }}
    }}

    pub const fn as_str(&self) -> &'static str {{
        match self {{
"
    )
    .unwrap();

    for keyword in primitives.keys() {
        writeln!(
            os,
            "            Self::{} => {:?},",
            to_camel_case_identifier(keyword),
            keyword,
        )
        .unwrap();
    }

    write!(
        os,
        "        }}
    }}

    pub const fn bits_of(&self) -> usize {{
        match self {{
"
    )
    .unwrap();

    for type_desc in primitives.values() {
        writeln!(
            os,
            "            Self::{} => {},",
            type_desc.identifier, type_desc.bits
        )
        .unwrap();
    }

    write!(
        os,
        "        }}
    }}

    pub const fn size_of(&self) -> usize {{
        match self {{
"
    )
    .unwrap();

    for type_desc in primitives.values() {
        writeln!(
            os,
            "            Self::{} => {},",
            type_desc.identifier, type_desc.size_of
        )
        .unwrap();
    }

    write!(
        os,
        "        }}
    }}

    pub const fn align_of(&self) -> usize {{
        match self {{
"
    )
    .unwrap();

    for type_desc in primitives.values() {
        writeln!(
            os,
            "            Self::{} => {},",
            type_desc.identifier, type_desc.align_of
        )
        .unwrap();
    }

    write!(
        os,
        "        }}
    }}

    pub const fn int_for_bits(bits: usize) -> Result<Self, ()> {{
        match bits {{
"
    )
    .unwrap();

    for type_desc in primitives.values().filter(|v| v.kind == TypeKind::Integer) {
        writeln!(
            os,
            "            {} => Ok(Self::{}),",
            type_desc.bits, type_desc.identifier,
        )
        .unwrap();
    }

    write!(
        os,
        "            _ => Err(())
        }}
    }}

    pub const fn uint_for_bits(bits: usize) -> Result<Self, ()> {{
        match bits {{
"
    )
    .unwrap();

    for type_desc in primitives.values().filter(|v| v.kind == TypeKind::Unsigned) {
        writeln!(
            os,
            "            {} => Ok(Self::{}),",
            type_desc.bits, type_desc.identifier,
        )
        .unwrap();
    }

    write!(
        os,
        "            _ => Err(())
        }}
    }}

    pub const fn is_signed(&self) -> bool {{
        match self {{
"
    )
    .unwrap();

    for type_desc in primitives
        .values()
        .filter(|v| matches!(v.kind, TypeKind::Integer | TypeKind::Float))
    {
        writeln!(os, "            Self::{} => true,", type_desc.identifier,).unwrap();
    }

    write!(
        os,
        "            _ => false
        }}
    }}

}}

impl core::fmt::Display for {class_name} {{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {{
        write!(f, \"{{}}\", self.as_str())
    }}
}}

impl core::fmt::Debug for {class_name} {{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {{
        write!(f, \"{{:?}}\", self.as_str())
    }}
}}
"
    )
    .unwrap();

    println!("cargo:rerun-if-changed={}", src_path);
}

fn make_irop(src_path: &str, dest_path: &str, class_name: &str, comment: &str) {
    let mut keywords = Vec::new();
    for line in read_to_string(src_path).unwrap().lines().skip(1) {
        let mut cols = line.split(",");
        let keyword = cols.next().unwrap().to_owned();
        if !keyword.is_empty() && !keyword.starts_with("#") {
            if keywords.contains(&keyword) {
                panic!("redefined keyword: {keyword}");
            }
            keywords.push(keyword);
        }
    }
    // keywords.sort();

    let mut os = File::create(dest_path).unwrap();

    write!(
        os,
        "//! {comment}

/* This file is generated automatically. DO NOT EDIT DIRECTLY. */

/// {comment}
#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum {class_name} {{
"
    )
    .unwrap();
    for keyword in keywords.iter() {
        writeln!(os, "    /// \"{}\"", keyword).unwrap();
        writeln!(os, "    {},", to_camel_case_identifier(keyword)).unwrap();
    }
    write!(
        os,
        "}}

impl {class_name} {{
    pub fn all_values() -> &'static [Self] {{
        &[
"
    )
    .unwrap();

    for keyword in keywords.iter() {
        writeln!(
            os,
            "            Self::{},",
            to_camel_case_identifier(keyword),
        )
        .unwrap();
    }

    write!(
        os,
        "        ]
    }}

    pub fn from_str(v: &str) -> Option<Self> {{
        match v {{
"
    )
    .unwrap();

    for keyword in keywords.iter() {
        writeln!(
            os,
            "            {:?} => Some(Self::{}),",
            keyword,
            to_camel_case_identifier(keyword),
        )
        .unwrap();
    }

    write!(
        os,
        "            _ => None,
        }}
    }}

    pub fn as_str(&self) -> &'static str {{
        match self {{
"
    )
    .unwrap();

    for keyword in keywords.iter() {
        writeln!(
            os,
            "            Self::{} => {:?},",
            to_camel_case_identifier(keyword),
            keyword,
        )
        .unwrap();
    }

    write!(
        os,
        "        }}
    }}
}}

impl core::fmt::Display for {class_name} {{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {{
        write!(f, \"{{}}\", self.as_str())
    }}
}}

impl core::fmt::Debug for {class_name} {{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {{
        write!(f, \"{{:?}}\", self.as_str())
    }}
}}
"
    )
    .unwrap();

    println!("cargo:rerun-if-changed={}", src_path);
}

fn to_camel_case_identifier(s: &str) -> String {
    let mut output = Vec::new();
    let mut flip = true;
    for ch in s.chars() {
        match ch {
            'A'..='Z' | '0'..='9' => {
                output.push(ch);
                flip = false;
            }
            'a'..='z' => {
                if flip {
                    output.push(ch.to_ascii_uppercase())
                } else {
                    output.push(ch);
                }
                flip = false;
            }
            _ => {
                flip = true;
            }
        }
    }

    output.into_iter().collect::<String>()
}
