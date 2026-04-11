use std::{borrow::Cow, ops::Range, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Pattern {
    string: Cow<'static, str>,
    segments: Cow<'static, [Range<usize>]>,
}

impl Pattern {
    pub fn new(
        string: impl Into<Cow<'static, str>>,
        segments: impl Into<Cow<'static, [Range<usize>]>>,
    ) -> Self {
        Self {
            string: string.into(),
            segments: segments.into(),
        }
    }
}

pub enum ParsePatternError {
    /// Path pattern must be either empty or start with a leading slash.
    LeadingSlash,
}

impl FromStr for Pattern {
    type Err = ParsePatternError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self::new("", &[]));
        }
        if !s.starts_with("/") {
            return Err(ParsePatternError::LeadingSlash);
        }

        Ok(Self::new(s.to_owned().into(), segments))
    }
}
