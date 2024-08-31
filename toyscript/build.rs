use std::{
    fs::{read_to_string, File},
    io::*,
};

fn main() {
    {
        make_enum(
            "./src/keyword/keyword.txt",
            "./src/keyword/keyword.rs",
            "Keyword",
            "ToyScript Reserved Keywords",
            &[],
        );
    }
}

fn make_enum(
    src_path: &str,
    dest_path: &str,
    class_name: &str,
    comment: &str,
    appending: &[String],
) {
    let mut keywords = Vec::new();
    for line in read_to_string(src_path).unwrap().lines() {
        let keyword = line.trim().to_string();
        if !keyword.is_empty() && !keyword.starts_with("#") {
            if keywords.contains(&keyword) {
                panic!("redefined keyword: {keyword}");
            }
            keywords.push(keyword);
        }
    }
    keywords.extend_from_slice(appending);
    keywords.sort();

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
