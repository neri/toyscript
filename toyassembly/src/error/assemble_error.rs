use crate::*;
use types::ValType;

#[derive(Debug)]
#[allow(unused)]
pub struct AssembleError {
    pub(crate) kind: AssembleErrorKind,
    pub(crate) explanation: Option<String>,
    pub(crate) position: ErrorPosition,
}

#[derive(Debug)]
pub enum AssembleErrorKind {
    SyntaxError,
    NameError,
    TypeMismatch,
    OutOfBounds,

    InternalError,
}

#[derive(Debug, PartialEq)]
pub enum ErrorPosition {
    CharAt(usize, usize),
    Unspecified,
}

impl AssembleError {
    #[inline]
    pub fn with_kind(kind: AssembleErrorKind, position: ErrorPosition) -> Self {
        Self {
            kind,
            explanation: None,
            position,
        }
    }

    pub fn invalid_identifier(source: &str, position: ErrorPosition) -> Self {
        let explanation = Some(format!("Invalid identifier: {:?}", source));
        AssembleError {
            kind: AssembleErrorKind::SyntaxError,
            explanation,
            position,
        }
    }

    pub fn undefined_identifier(source: &str, position: ErrorPosition) -> Self {
        let explanation = Some(format!("Not found in this scope: {:?}", source));
        AssembleError {
            kind: AssembleErrorKind::NameError,
            explanation,
            position,
        }
    }

    pub fn duplicated_identifier(identifier: &str, position: ErrorPosition) -> Self {
        let explanation = Some(format!(
            "The identifier {:?} has already been defined",
            identifier
        ));
        AssembleError {
            kind: AssembleErrorKind::NameError,
            explanation,
            position: position,
        }
    }

    #[inline]
    pub fn out_of_bounds(explain: &str, position: ErrorPosition) -> Self {
        AssembleError {
            kind: AssembleErrorKind::OutOfBounds,
            explanation: Some(explain.to_owned()),
            position,
        }
    }

    pub fn check_types(
        expected: &[ValType],
        actual: &[ValType],
        position: ErrorPosition,
    ) -> Result<(), AssembleError> {
        (expected == actual)
            .then(|| ())
            .ok_or_else(|| AssembleError::type_mismatch(expected, actual, position))
    }

    #[inline]
    pub fn type_mismatch(
        expected: &[ValType],
        actual: &[ValType],
        position: ErrorPosition,
    ) -> Self {
        let explanation = Some(format!(
            "Type mismatch {}. expected {}",
            Self::explanation_valtype_strings(actual),
            Self::explanation_valtype_strings(expected),
        ));
        AssembleError {
            kind: AssembleErrorKind::TypeMismatch,
            explanation,
            position,
        }
    }

    #[inline]
    pub fn internal_inconsistency(explain: &str, position: ErrorPosition) -> Self {
        AssembleError {
            kind: AssembleErrorKind::InternalError,
            explanation: (explain.len() > 0).then(|| explain.to_owned()),
            position,
        }
    }

    pub fn explanation_valtype_strings(types: &[ValType]) -> String {
        if types.len() > 0 {
            let mut buf = Vec::new();
            for valtype in types {
                buf.push(valtype.as_str());
            }
            format!("({})", buf.join(", "))
        } else {
            "empty".to_owned()
        }
    }

    #[inline]
    pub fn explanation(&self) -> Option<&str> {
        self.explanation.as_ref().map(|v| v.as_str())
    }

    #[inline]
    pub fn position(&self) -> &ErrorPosition {
        &self.position
    }
}
