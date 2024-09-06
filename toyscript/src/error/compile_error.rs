use crate::{ast::identifier::Identifier, *};
use core::cmp;
use tir::{opt::OptimizeError, AssembleError};
use token::{TokenError, TokenPosition};

#[derive(Debug)]
pub struct CompileError {
    kind: CompileErrorKind,
    explanation: Option<String>,
    position: ErrorPosition,
}

#[derive(Debug, PartialEq)]
pub enum ErrorPosition {
    CharAt(usize, usize),
    Range(TokenPosition),
    Unspecified,
}

#[derive(Debug)]
pub enum CompileErrorKind {
    TokenParseError(TokenError),

    SyntaxError,
    InvalidLiteral,
    DuplicateIdentifier,
    IdentifierNotFound,
    TypeError,
    InvalidType,

    InternalError,
}

impl CompileError {
    #[inline]
    pub fn new(kind: CompileErrorKind, line: usize, column: usize) -> Self {
        Self {
            kind,
            explanation: None,
            position: ErrorPosition::CharAt(line, column),
        }
    }

    #[inline]
    pub fn with_kind(kind: CompileErrorKind, position: ErrorPosition) -> Self {
        Self {
            kind,
            explanation: None,
            position,
        }
    }

    #[inline]
    pub fn with_explanation(
        kind: CompileErrorKind,
        line: usize,
        column: usize,
        explanation: String,
    ) -> Self {
        Self {
            kind,
            explanation: Some(explanation),
            position: ErrorPosition::CharAt(line, column),
        }
    }

    #[inline]
    pub fn with_token(
        kind: CompileErrorKind,
        token: &Token<Keyword>,
        explanation: Option<String>,
    ) -> Self {
        Self {
            kind,
            explanation,
            position: token.position().into(),
        }
    }

    #[inline]
    pub fn with_token_opt(
        kind: CompileErrorKind,
        token: Option<&Token<Keyword>>,
        explanation: Option<String>,
    ) -> Self {
        if let Some(token) = token {
            Self::with_token(kind, token, explanation)
        } else {
            Self {
                kind,
                explanation,
                position: ErrorPosition::CharAt(0, 0),
            }
        }
    }

    #[inline]
    pub fn with_position(
        kind: CompileErrorKind,
        position: TokenPosition,
        explanation: Option<String>,
    ) -> Self {
        Self {
            kind,
            explanation,
            position: position.into(),
        }
    }

    #[inline]
    pub fn internal_inconsistency(explanation: &str, position: ErrorPosition) -> Self {
        let explanation = format!("Internal inconsistency: {}", explanation);
        Self {
            kind: CompileErrorKind::InternalError,
            explanation: Some(explanation),
            position,
        }
    }

    #[inline]
    pub fn unexpected_token(token: &Token<Keyword>) -> Self {
        let explanation = format!(
            "Unexpected token {}",
            Self::explanation_token_strings(&[*token.token_type()]).unwrap(),
        );
        Self::with_token(CompileErrorKind::SyntaxError, token, Some(explanation))
    }

    #[inline]
    pub fn missing_token(token: &Token<Keyword>, expected: &[TokenType<Keyword>]) -> Self {
        let explanation = Self::explanation_token_strings(expected).map(|v| {
            format!(
                "Unexpected token {}. Expected {}",
                Self::explanation_token_strings(&[*token.token_type()]).unwrap(),
                v,
            )
        });
        Self::with_token(CompileErrorKind::SyntaxError, token, explanation)
    }

    #[inline]
    pub fn missing_eol(token: &Token<Keyword>) -> Self {
        let explanation = format!(
            "Unexpected token {}. Expected EndOfLine",
            Self::explanation_token_strings(&[*token.token_type()]).unwrap()
        );
        Self::with_token(CompileErrorKind::SyntaxError, token, Some(explanation))
    }

    #[inline]
    pub fn invalid_literal(explanation: &str, position: TokenPosition) -> Self {
        Self::with_position(
            CompileErrorKind::InvalidLiteral,
            position,
            Some(explanation.to_owned()),
        )
    }

    #[inline]
    pub fn out_of_context(explanation: &str, position: TokenPosition) -> Self {
        let explanation = format!(
            "This statement cannot be used in the current context: {}",
            explanation
        );
        Self::with_position(CompileErrorKind::SyntaxError, position, Some(explanation))
    }

    #[inline]
    pub fn out_of_value_stack(position: TokenPosition) -> Self {
        let explanation = format!("Out of Value Stack");
        Self::with_position(CompileErrorKind::InternalError, position, Some(explanation))
    }

    #[inline]
    pub fn duplicate_identifier(identifier: &Identifier) -> Self {
        let explanation = format!(
            "The identifier '{}' is duplicate defined in the current scope.",
            identifier.as_str()
        );
        Self::with_position(
            CompileErrorKind::DuplicateIdentifier,
            identifier.id_position(),
            Some(explanation),
        )
    }

    #[inline]
    pub fn identifier_not_found(identifier: &Identifier) -> Self {
        let explanation = format!(
            "Identifier '{}' is not defined in the current scope.",
            identifier
        );
        Self::with_position(
            CompileErrorKind::IdentifierNotFound,
            identifier.id_position(),
            Some(explanation),
        )
    }

    #[inline]
    pub fn identifier_not_found_looping(identifier: &Identifier) -> Self {
        let explanation = format!(
            "Identifier not found '{}'. Maybe the reference is looping or too complex.",
            identifier.as_str()
        );
        Self::with_position(
            CompileErrorKind::IdentifierNotFound,
            identifier.id_position(),
            Some(explanation),
        )
    }

    #[inline]
    pub fn type_mismatch(
        exptected: &types::TypeDescriptor,
        actual: &types::TypeDescriptor,
        position: TokenPosition,
    ) -> Self {
        let explanation = format!(
            "Type mismatch '{}', Expected '{}'",
            actual.identifier(),
            exptected.identifier(),
        );
        Self::with_position(CompileErrorKind::TypeError, position, Some(explanation))
    }

    #[inline]
    pub fn literal_overflow(exptected: &types::TypeDescriptor, position: TokenPosition) -> Self {
        let explanation = format!("Literal out of range for '{}' ", exptected.identifier(),);
        Self::with_position(CompileErrorKind::TypeError, position, Some(explanation))
    }

    #[inline]
    pub fn invalid_type(identifier: &Identifier) -> Self {
        let explanation = format!("Invalid use of type `{}`", identifier.as_str());
        Self::with_position(
            CompileErrorKind::TypeError,
            identifier.id_position(),
            Some(explanation),
        )
    }

    #[inline]
    pub fn const_out_of_range(value: String, position: TokenPosition) -> Self {
        let explanation = format!("Constant value '{}' is Out of range", value);
        Self::with_position(CompileErrorKind::TypeError, position, Some(explanation))
    }

    #[inline]
    pub fn return_required(exptected: &types::TypeDescriptor, position: TokenPosition) -> Self {
        let explanation = format!(
            "Requires a return value of type '{}'",
            exptected.identifier(),
        );
        Self::with_position(CompileErrorKind::TypeError, position, Some(explanation))
    }

    #[inline]
    pub fn lvalue_required(position: TokenPosition) -> Self {
        let explanation = format!("Lvalue required as left operand of assignment.");
        Self {
            kind: CompileErrorKind::SyntaxError,
            explanation: Some(explanation),
            position: ErrorPosition::Range(position),
        }
    }

    #[inline]
    pub fn cannot_assign(identifier: &Identifier) -> Self {
        let explanation = format!(
            "Cannot assign to '{}', it is not mutable.",
            identifier.as_str()
        );
        Self {
            kind: CompileErrorKind::SyntaxError,
            explanation: Some(explanation),
            position: ErrorPosition::Range(identifier.id_position()),
        }
    }

    #[inline]
    pub fn could_not_infer(identifier: &Identifier) -> Self {
        let explanation = format!("Could not infer type of '{}'", identifier.as_str());
        Self {
            kind: CompileErrorKind::SyntaxError,
            explanation: Some(explanation),
            position: ErrorPosition::Range(identifier.id_position()),
        }
    }

    #[inline]
    pub fn function_parameter_number_mismatch(
        expected: usize,
        actual: usize,
        position: TokenPosition,
    ) -> Self {
        let explanation = format!(
            "this function takes {} arguments but {} arguments was supplied",
            expected, actual
        );
        Self {
            kind: CompileErrorKind::SyntaxError,
            explanation: Some(explanation),
            position: ErrorPosition::Range(position),
        }
    }

    #[inline]
    pub fn todo(message: Option<String>, position: TokenPosition) -> Self {
        let explanation = if let Some(message) = message {
            format!("Not yet implemented: {}", message)
        } else {
            "Not yet implemented".to_string()
        };

        Self {
            kind: CompileErrorKind::InternalError,
            explanation: Some(explanation),
            position: ErrorPosition::Range(position),
        }
    }

    #[inline]
    pub fn kind(&self) -> &CompileErrorKind {
        &self.kind
    }

    #[inline]
    pub fn explanation(&self) -> Option<&str> {
        self.explanation.as_ref().map(|v| v.as_str())
    }

    #[inline]
    pub fn position(&self) -> &ErrorPosition {
        &self.position
    }

    pub fn explanation_token_strings(types: &[TokenType<Keyword>]) -> Option<String> {
        match types.len() {
            0 => None,
            1 => Some(format!("{}", types[0])),
            len => {
                let mut tokens = types.iter().map(|v| format!("{}", v)).collect::<Vec<_>>();
                tokens.sort();
                let tokens = if len > 2 {
                    let (last, tokens) = tokens.split_last().unwrap();
                    [tokens.join(", "), last.clone()].join(" and ")
                } else {
                    tokens.join(" and ")
                };
                Some(tokens)
            }
        }
    }

    pub fn to_detail_string(
        &self,
        file_name: &str,
        blob: &[u8],
        line_positions: &[(usize, usize)],
    ) -> String {
        let mut lines = Vec::new();
        lines.push(format!("error: {:?}", self.kind));

        match self.position() {
            ErrorPosition::Unspecified => {
                lines.push(format!("{}:", file_name));
            }
            ErrorPosition::CharAt(line, column) => {
                if *line > 0 && *column > 0 {
                    lines.push(format!("{}:{}:{}", file_name, line, column));
                }
            }
            ErrorPosition::Range(position) => {
                let range_start = position.start();
                let range_end = position.end();

                if let Ok(line_index) = line_positions.binary_search_by(|(line_start, line_end)| {
                    if range_start < *line_start {
                        cmp::Ordering::Greater
                    } else if range_start > *line_end {
                        cmp::Ordering::Less
                    } else {
                        cmp::Ordering::Equal
                    }
                }) {
                    let position = line_positions[line_index];
                    let line = line_index + 1;
                    let column = range_start - position.0 + 1;

                    lines.push(format!("{}:{}:{}", file_name, line, column));

                    if let Some(line) = blob
                        .get(position.0..position.1)
                        .and_then(|v| str::from_utf8(v).ok())
                    {
                        lines.push(format!("\n{}", line));

                        let leading = if column > 1 {
                            " ".repeat(column - 1)
                        } else {
                            "".to_string()
                        };
                        let cursor_end = range_end.min(position.1);
                        let cursor_len = cursor_end.saturating_sub(range_start);
                        let cursor = if cursor_len > 1 {
                            "^".repeat(cursor_len)
                        } else {
                            "^".to_string()
                        };

                        lines.push(format!("{}{}", leading, cursor));
                    }
                }
            }
        }

        if let Some(explanation) = self.explanation() {
            lines.push(format!("\n{}", explanation));
        }

        lines.join("\n")
    }
}

impl From<TokenPosition> for ErrorPosition {
    #[inline]
    fn from(value: TokenPosition) -> Self {
        ErrorPosition::Range(value)
    }
}

impl From<AssembleError> for CompileError {
    fn from(err: AssembleError) -> Self {
        CompileError::internal_inconsistency(
            &format!("Internal Assembler Error: {:?}", err),
            ErrorPosition::Unspecified,
        )
    }
}

impl From<OptimizeError> for CompileError {
    fn from(err: OptimizeError) -> Self {
        CompileError::internal_inconsistency(
            &format!("Code Optimization Error: {:?}", err),
            ErrorPosition::Unspecified,
        )
    }
}
