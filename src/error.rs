use std::{error::Error, fmt};

#[derive(Debug)]
pub enum NBPCError {
    PaddingCorruption(usize),
    OverflowingModification,
    NoInputPMFs,
    #[cfg(feature = "args_validation")]
    IncorrectInput(String),
}

impl fmt::Display for NBPCError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PaddingCorruption(n) => f.write_str(&format!("{n} padding bits were modified")),
            Self::OverflowingModification => f.write_str("under/overflowing modification happened"),
            Self::NoInputPMFs => {
                f.write_str("can't calculate input PMFs for given costs and payload")
            }

            #[cfg(feature = "args_validation")]
            Self::IncorrectInput(s) => f.write_str(&format!("incorrect input: {s}")),
        }
    }
}

impl Error for NBPCError {}
