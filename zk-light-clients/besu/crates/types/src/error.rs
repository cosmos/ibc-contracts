use core::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TypeError {
    UnsupportedSchema { actual: u32 },
    ZeroHeight,
    ZeroAddress,
    EmptyCollection(&'static str),
}

impl fmt::Display for TypeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchema { actual } => {
                write!(formatter, "unsupported schema version {actual}")
            }
            Self::ZeroHeight => formatter.write_str("height must be non-zero"),
            Self::ZeroAddress => formatter.write_str("address must be non-zero"),
            Self::EmptyCollection(name) => write!(formatter, "{name} must not be empty"),
        }
    }
}

impl core::error::Error for TypeError {}
