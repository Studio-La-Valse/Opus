use serde::Deserialize;
use std::fmt;
use std::str::FromStr;

/// Deserializes from the same `"horizontal"` / `"vertical"` strings [`FromStr`]
/// accepts, so a CLI flag, a CSS custom property and a JSON option all spell it
/// the same way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(try_from = "String")]
pub enum PageOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug)]
pub struct PageOrientationParseError(String);

impl fmt::Display for PageOrientationParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid page orientation '{}': expected 'horizontal' or 'vertical'",
            self.0
        )
    }
}

impl std::error::Error for PageOrientationParseError {}

impl FromStr for PageOrientation {
    type Err = PageOrientationParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "horizontal" => Ok(PageOrientation::Horizontal),
            "vertical" => Ok(PageOrientation::Vertical),
            _ => Err(PageOrientationParseError(s.to_string())),
        }
    }
}

/// Backs `#[serde(try_from = "String")]` on [`PageOrientation`].
impl TryFrom<String> for PageOrientation {
    type Error = PageOrientationParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}
