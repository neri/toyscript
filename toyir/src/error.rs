use core::error::Error;
use core::fmt;

pub enum AssembleError {
    InvalidPrimitive,
    InvalidParameter,
    OutOfBlockStack,
    OutOfValueStack,
    InvalidBlockStack,
    InvalidValueStack,
    InvalidBranchTarget,
}

impl Error for AssembleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }

    //fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {}
}

impl fmt::Debug for AssembleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for AssembleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrimitive => write!(f, "InvalidPrimitive"),
            Self::InvalidParameter => write!(f, "InvalidParameter"),
            Self::OutOfBlockStack => write!(f, "OutOfBlockStack"),
            Self::OutOfValueStack => write!(f, "OutOfValueStack"),
            Self::InvalidBlockStack => write!(f, "InvalidBlockStack"),
            Self::InvalidValueStack => write!(f, "InvalidValueStack"),
            Self::InvalidBranchTarget => write!(f, "InvalidBranchTarget"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum OptimizeError {
    OutOfPosition(usize),

    OutOfCodes(usize),

    InvalidBranch(usize, u32),

    InvalidParameter(usize, usize),

    OverwriteError(usize, usize, usize),

    InvalidDropChain(usize),

    RenameError(usize, usize, usize),

    TypeCastError(usize, u32),
}

impl Error for OptimizeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }

    // fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {}
}

impl fmt::Debug for OptimizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for OptimizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfPosition(arg0) => f.debug_tuple("OutOfPosition").field(arg0).finish(),
            Self::OutOfCodes(arg0) => f.debug_tuple("OutOfCodes").field(arg0).finish(),
            Self::InvalidBranch(arg0, arg1) => f
                .debug_tuple("InvalidBranch")
                .field(arg0)
                .field(arg1)
                .finish(),
            Self::InvalidParameter(arg0, arg1) => f
                .debug_tuple("InvalidParameter")
                .field(arg0)
                .field(arg1)
                .finish(),
            Self::OverwriteError(arg0, arg1, arg2) => f
                .debug_tuple("OverwriteError")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::InvalidDropChain(arg0) => f.debug_tuple("InvalidDropChain").field(arg0).finish(),
            Self::RenameError(arg0, arg1, arg2) => f
                .debug_tuple("RenameError")
                .field(arg0)
                .field(arg1)
                .field(arg2)
                .finish(),
            Self::TypeCastError(arg0, arg1) => f
                .debug_tuple("TypeCastError")
                .field(arg0)
                .field(arg1)
                .finish(),
        }
    }
}
