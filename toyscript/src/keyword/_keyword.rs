//! ToyScript Reserved Keywords

/* This file is generated automatically. DO NOT EDIT DIRECTLY. */

/// ToyScript Reserved Keywords
#[non_exhaustive]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Keyword {
    /// "any"
    Any,
    /// "as"
    As,
    /// "async"
    Async,
    /// "await"
    Await,
    /// "boolean"
    Boolean,
    /// "break"
    Break,
    /// "case"
    Case,
    /// "catch"
    Catch,
    /// "class"
    Class,
    /// "const"
    Const,
    /// "constructor"
    Constructor,
    /// "continue"
    Continue,
    /// "debugger"
    Debugger,
    /// "declare"
    Declare,
    /// "default"
    Default,
    /// "delete"
    Delete,
    /// "do"
    Do,
    /// "else"
    Else,
    /// "enum"
    Enum,
    /// "export"
    Export,
    /// "extends"
    Extends,
    /// "false"
    False,
    /// "finally"
    Finally,
    /// "for"
    For,
    /// "from"
    From,
    /// "function"
    Function,
    /// "get"
    Get,
    /// "if"
    If,
    /// "implements"
    Implements,
    /// "import"
    Import,
    /// "in"
    In,
    /// "instanceof"
    Instanceof,
    /// "interface"
    Interface,
    /// "let"
    Let,
    /// "module"
    Module,
    /// "namespace"
    Namespace,
    /// "new"
    New,
    /// "null"
    Null,
    /// "number"
    Number,
    /// "of"
    Of,
    /// "package"
    Package,
    /// "private"
    Private,
    /// "protected"
    Protected,
    /// "public"
    Public,
    /// "require"
    Require,
    /// "return"
    Return,
    /// "set"
    Set,
    /// "static"
    Static,
    /// "string"
    String,
    /// "super"
    Super,
    /// "switch"
    Switch,
    /// "symbol"
    Symbol,
    /// "this"
    This,
    /// "throw"
    Throw,
    /// "true"
    True,
    /// "try"
    Try,
    /// "type"
    Type,
    /// "typeof"
    Typeof,
    /// "undefined"
    Undefined,
    /// "var"
    Var,
    /// "void"
    Void,
    /// "while"
    While,
    /// "with"
    With,
    /// "yield"
    Yield,
}

impl Keyword {
    pub fn all_values() -> &'static [Self] {
        &[
            Self::Any,
            Self::As,
            Self::Async,
            Self::Await,
            Self::Boolean,
            Self::Break,
            Self::Case,
            Self::Catch,
            Self::Class,
            Self::Const,
            Self::Constructor,
            Self::Continue,
            Self::Debugger,
            Self::Declare,
            Self::Default,
            Self::Delete,
            Self::Do,
            Self::Else,
            Self::Enum,
            Self::Export,
            Self::Extends,
            Self::False,
            Self::Finally,
            Self::For,
            Self::From,
            Self::Function,
            Self::Get,
            Self::If,
            Self::Implements,
            Self::Import,
            Self::In,
            Self::Instanceof,
            Self::Interface,
            Self::Let,
            Self::Module,
            Self::Namespace,
            Self::New,
            Self::Null,
            Self::Number,
            Self::Of,
            Self::Package,
            Self::Private,
            Self::Protected,
            Self::Public,
            Self::Require,
            Self::Return,
            Self::Set,
            Self::Static,
            Self::String,
            Self::Super,
            Self::Switch,
            Self::Symbol,
            Self::This,
            Self::Throw,
            Self::True,
            Self::Try,
            Self::Type,
            Self::Typeof,
            Self::Undefined,
            Self::Var,
            Self::Void,
            Self::While,
            Self::With,
            Self::Yield,
        ]
    }

    pub fn from_str(v: &str) -> Option<Self> {
        match v {
            "any" => Some(Self::Any),
            "as" => Some(Self::As),
            "async" => Some(Self::Async),
            "await" => Some(Self::Await),
            "boolean" => Some(Self::Boolean),
            "break" => Some(Self::Break),
            "case" => Some(Self::Case),
            "catch" => Some(Self::Catch),
            "class" => Some(Self::Class),
            "const" => Some(Self::Const),
            "constructor" => Some(Self::Constructor),
            "continue" => Some(Self::Continue),
            "debugger" => Some(Self::Debugger),
            "declare" => Some(Self::Declare),
            "default" => Some(Self::Default),
            "delete" => Some(Self::Delete),
            "do" => Some(Self::Do),
            "else" => Some(Self::Else),
            "enum" => Some(Self::Enum),
            "export" => Some(Self::Export),
            "extends" => Some(Self::Extends),
            "false" => Some(Self::False),
            "finally" => Some(Self::Finally),
            "for" => Some(Self::For),
            "from" => Some(Self::From),
            "function" => Some(Self::Function),
            "get" => Some(Self::Get),
            "if" => Some(Self::If),
            "implements" => Some(Self::Implements),
            "import" => Some(Self::Import),
            "in" => Some(Self::In),
            "instanceof" => Some(Self::Instanceof),
            "interface" => Some(Self::Interface),
            "let" => Some(Self::Let),
            "module" => Some(Self::Module),
            "namespace" => Some(Self::Namespace),
            "new" => Some(Self::New),
            "null" => Some(Self::Null),
            "number" => Some(Self::Number),
            "of" => Some(Self::Of),
            "package" => Some(Self::Package),
            "private" => Some(Self::Private),
            "protected" => Some(Self::Protected),
            "public" => Some(Self::Public),
            "require" => Some(Self::Require),
            "return" => Some(Self::Return),
            "set" => Some(Self::Set),
            "static" => Some(Self::Static),
            "string" => Some(Self::String),
            "super" => Some(Self::Super),
            "switch" => Some(Self::Switch),
            "symbol" => Some(Self::Symbol),
            "this" => Some(Self::This),
            "throw" => Some(Self::Throw),
            "true" => Some(Self::True),
            "try" => Some(Self::Try),
            "type" => Some(Self::Type),
            "typeof" => Some(Self::Typeof),
            "undefined" => Some(Self::Undefined),
            "var" => Some(Self::Var),
            "void" => Some(Self::Void),
            "while" => Some(Self::While),
            "with" => Some(Self::With),
            "yield" => Some(Self::Yield),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::As => "as",
            Self::Async => "async",
            Self::Await => "await",
            Self::Boolean => "boolean",
            Self::Break => "break",
            Self::Case => "case",
            Self::Catch => "catch",
            Self::Class => "class",
            Self::Const => "const",
            Self::Constructor => "constructor",
            Self::Continue => "continue",
            Self::Debugger => "debugger",
            Self::Declare => "declare",
            Self::Default => "default",
            Self::Delete => "delete",
            Self::Do => "do",
            Self::Else => "else",
            Self::Enum => "enum",
            Self::Export => "export",
            Self::Extends => "extends",
            Self::False => "false",
            Self::Finally => "finally",
            Self::For => "for",
            Self::From => "from",
            Self::Function => "function",
            Self::Get => "get",
            Self::If => "if",
            Self::Implements => "implements",
            Self::Import => "import",
            Self::In => "in",
            Self::Instanceof => "instanceof",
            Self::Interface => "interface",
            Self::Let => "let",
            Self::Module => "module",
            Self::Namespace => "namespace",
            Self::New => "new",
            Self::Null => "null",
            Self::Number => "number",
            Self::Of => "of",
            Self::Package => "package",
            Self::Private => "private",
            Self::Protected => "protected",
            Self::Public => "public",
            Self::Require => "require",
            Self::Return => "return",
            Self::Set => "set",
            Self::Static => "static",
            Self::String => "string",
            Self::Super => "super",
            Self::Switch => "switch",
            Self::Symbol => "symbol",
            Self::This => "this",
            Self::Throw => "throw",
            Self::True => "true",
            Self::Try => "try",
            Self::Type => "type",
            Self::Typeof => "typeof",
            Self::Undefined => "undefined",
            Self::Var => "var",
            Self::Void => "void",
            Self::While => "while",
            Self::With => "with",
            Self::Yield => "yield",
        }
    }
}

impl core::fmt::Display for Keyword {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl core::fmt::Debug for Keyword {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Keyword({})", self.as_str())
    }
}
