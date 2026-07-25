#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accidental {
    Natural,
    Sharp,
    DoubleSharp,
    Flat,
    DoubleFlat,
}

impl TryFrom<&str> for Accidental {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "natural" => Ok(Accidental::Natural),
            "sharp" => Ok(Accidental::Sharp),
            "double-sharp" => Ok(Accidental::DoubleSharp),
            "sharp-sharp" => Ok(Accidental::DoubleSharp),
            "flat" => Ok(Accidental::Flat),
            "flat-flat" => Ok(Accidental::DoubleFlat),
            other => Err(format!("{} is not an Alter", other)),
        }
    }
}
