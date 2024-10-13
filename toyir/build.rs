use std::{
    collections::BTreeMap,
    fs::{read_to_string, File},
    io::*,
};

fn main() {
    {
        make_primitive(
            "./src/misc/primitive.csv",
            "./src/_generated/primitive.rs",
            "./src/_generated/opt_cast.rs",
            "Primitive",
            "ToyScript Primitive Types",
        );
    }

    {
        make_irop(
            "./src/misc/irop.csv",
            "./src/_generated/irop.rs",
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
    type_id: u32,
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

//
fn make_primitive(
    src_path: &str,
    dest_path: &str,
    dest_path_opt: &str,
    class_name: &str,
    comment: &str,
) {
    let mut primitives = BTreeMap::<String, PrimitiveTypeDesc>::new();

    for line in read_to_string(src_path).unwrap().lines().skip(1) {
        if line.starts_with("#") {
            continue;
        }
        let mut cols = line.split(",");
        let name = cols.next().unwrap().to_owned();
        let bits = cols.next().unwrap().parse::<usize>().unwrap();
        let size_of = cols.next().unwrap().parse::<usize>().unwrap();
        let align_of = cols.next().unwrap().parse::<usize>().unwrap();
        let kind = TypeKind::from_str(cols.next().unwrap()).unwrap();

        let is_unsigned = matches!(kind, TypeKind::Unsigned);
        let is_float = matches!(kind, TypeKind::Float);
        let type_id = if matches!(kind, TypeKind::Void) {
            0
        } else {
            (is_unsigned as u32) + ((is_float as u32) << 1) + ((size_of as u32) << 2)
        };

        let primitive = PrimitiveTypeDesc {
            name: name.clone(),
            identifier: to_camel_case_identifier(&name),
            bits,
            size_of,
            align_of,
            kind,
            type_id,
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

    let mut sorted_primitives = primitives.values().collect::<Vec<_>>();
    sorted_primitives.sort_by(|a, b| a.type_id.cmp(&b.type_id));

    let mut os = File::create(dest_path).unwrap();

    write!(
        os,
        "//! {comment}

/* This file is generated automatically. DO NOT EDIT DIRECTLY. */

/// {comment}
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum {class_name} {{
"
    )
    .unwrap();
    for type_desc in sorted_primitives.iter() {
        writeln!(os, "    /// \"{}\"", type_desc.name).unwrap();
        writeln!(
            os,
            "    {} = {:#04x},",
            to_camel_case_identifier(&type_desc.name),
            type_desc.type_id
        )
        .unwrap();
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

    for type_desc in sorted_primitives.iter() {
        writeln!(
            os,
            "            Self::{},",
            to_camel_case_identifier(&type_desc.name),
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

    for type_desc in sorted_primitives.iter() {
        writeln!(
            os,
            "            Self::{} => {:?},",
            to_camel_case_identifier(&type_desc.name),
            type_desc.name,
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

    for type_desc in sorted_primitives.iter() {
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

    for type_desc in sorted_primitives.iter() {
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

    for type_desc in sorted_primitives.iter() {
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

    for type_desc in sorted_primitives
        .iter()
        .filter(|v| matches!(v.kind, TypeKind::Integer | TypeKind::Float))
    {
        writeln!(os, "            Self::{} => true,", type_desc.identifier,).unwrap();
    }

    write!(
        os,
        "            _ => false
        }}
    }}

    pub const fn is_integer(&self) -> bool {{
        match self {{
"
    )
    .unwrap();

    for type_desc in sorted_primitives
        .iter()
        .filter(|v| matches!(v.kind, TypeKind::Integer | TypeKind::Unsigned))
    {
        writeln!(os, "            Self::{} => true,", type_desc.identifier,).unwrap();
    }

    write!(
        os,
        "            _ => false
        }}
    }}

    pub const fn is_float(&self) -> bool {{
        match self {{
"
    )
    .unwrap();

    for type_desc in sorted_primitives
        .iter()
        .filter(|v| matches!(v.kind, TypeKind::Float))
    {
        writeln!(os, "            Self::{} => true,", type_desc.identifier,).unwrap();
    }

    write!(
        os,
        "            _ => false
        }}
    }}

    pub fn storage_type(&self) -> Self {{
        match self {{
"
    )
    .unwrap();

    for type_desc in sorted_primitives.iter() {
        let dest_type = match type_desc.kind {
            TypeKind::Integer | TypeKind::Unsigned => {
                format!(
                    "I{}",
                    if type_desc.bits < 32 {
                        32
                    } else {
                        type_desc.bits
                    }
                )
            }
            TypeKind::Float | TypeKind::Void => type_desc.identifier.clone(),
        };

        writeln!(
            os,
            "            Self::{} => Self::{},",
            type_desc.identifier, dest_type,
        )
        .unwrap();
    }

    write!(
        os,
        "        }}
    }}

    /// type id
    ///   sum of:
    ///     (is_unsigned: 1)
    ///     (is_float: 2)
    ///     (size_of_type << 2)
    #[inline]
    pub const fn type_id(&self) -> u32 {{
        *self as u32
    }}

    pub fn from_type_id(v: u32) -> Option<Self> {{
        match v {{
"
    )
    .unwrap();

    let mut types = primitives
        .values()
        .filter(|v| v.type_id != 0)
        .collect::<Vec<_>>();
    types.sort_by(|a, b| a.type_id.cmp(&b.type_id));

    for type_desc in types.iter() {
        writeln!(
            os,
            "            {:#04x} => Some(Self::{}),",
            type_desc.type_id, type_desc.identifier,
        )
        .unwrap();
    }

    write!(
        os,
        "            _ => None
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

    drop(os);

    let mut os = File::create(dest_path_opt).unwrap();

    write!(
        os,
        "//! Optimize constant casts

/* This file is generated automatically. DO NOT EDIT DIRECTLY. */

use super::*;

#[inline]
pub(super) fn opt_cast(
    opt: &mut MinimalCodeOptimizer,
    old_type: Primitive, 
    new_type: Primitive, 
    const_val: Constant,
    base: ArrayIndex, 
    target: ArrayIndex, 
    result: CodeIndex,
) -> Result<bool, OptimizeError> {{
    match (old_type, new_type, const_val) {{
"
    )
    .unwrap();

    for src_type in types.iter() {
        for dest_type in types.iter() {
            let src_const_class = match src_type.identifier.as_str() {
                "I8" | "I16" | "U8" | "U16" | "U32" | "I32" => "I32",
                "I64" | "U64" => "I64",
                class => class,
            };
            let dest_const_class = match dest_type.identifier.as_str() {
                "I8" | "I16" | "U8" | "U16" | "U32" | "I32" => "I32",
                "I64" | "U64" => "I64",
                class => class,
            };

            write!(
                os,
                "        (Primitive::{}, Primitive::{}, Constant::{}(const_val)) => {{
            opt.replace_nop(target)?;
            opt.replace_{}_const(base, result, ((const_val as {}) as {}) as {})?;
            Ok(true)
        }},
",
                src_type.identifier,
                dest_type.identifier,
                src_const_class,
                dest_const_class.to_lowercase(),
                src_type.identifier.to_lowercase(),
                dest_type.identifier.to_lowercase(),
                dest_const_class.to_lowercase(),
            )
            .unwrap();
        }
    }

    write!(
        os,
        "        _ => todo!()
    }}
}}
"
    )
    .unwrap();

    println!("cargo:rerun-if-changed={}", src_path);
}

fn make_irop(src_path: &str, dest_path: &str, class_name: &str, comment: &str) {
    let kind_class_name = format!("{class_name}Class");

    let mut keyword_kind = BTreeMap::new();
    let mut keywords = Vec::new();
    let mut kinds = Vec::new();
    for line in read_to_string(src_path).unwrap().lines().skip(1) {
        let mut cols = line.split(",");
        let comment = cols.next().unwrap();
        if comment.starts_with("#") {
            continue;
        }
        let keyword = cols.next().unwrap().to_owned();
        if keyword.is_empty() {
            continue;
        }
        let kind = cols.next().unwrap().to_owned();

        if keywords.contains(&keyword) {
            panic!("redefined keyword: {keyword}");
        }

        keyword_kind.insert(keyword.clone(), kind.clone());
        keywords.push(keyword);

        if !kinds.contains(&kind) {
            kinds.push(kind);
        }
    }
    keywords.sort();

    let mut os = File::create(dest_path).unwrap();

    write!(
        os,
        "//! {comment}

/* This file is generated automatically. DO NOT EDIT DIRECTLY. */

/// {comment}
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

    pub fn to_identifier(&self) -> &str {{
        match self {{
"
    )
    .unwrap();

    for keyword in keywords.iter() {
        writeln!(
            os,
            "            Self::{} => {:?},",
            to_camel_case_identifier(keyword),
            to_camel_case_identifier(keyword),
        )
        .unwrap();
    }

    write!(
        os,
        "        }}
    }}

    pub fn class(&self) -> {kind_class_name} {{
        match self {{
"
    )
    .unwrap();

    for keyword in keywords.iter() {
        writeln!(
            os,
            "            Self::{} => {kind_class_name}::{},",
            to_camel_case_identifier(&keyword),
            to_camel_case_identifier(keyword_kind.get(keyword).unwrap()),
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum {kind_class_name} {{
"
    )
    .unwrap();

    for kind in &kinds {
        writeln!(os, "    {},", to_camel_case_identifier(kind)).unwrap();
    }

    writeln!(os, "}}").unwrap();

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
