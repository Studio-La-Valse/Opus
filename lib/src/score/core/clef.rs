use crate::score::core::pitch::Pitch;

#[derive(Clone)]
pub struct Clef {
    pub line_middle_c: i32,
    pub name: String,
}

impl Clef {
    pub fn parse(sign: &str, line: Option<i32>) -> Self {
        match sign.to_lowercase().as_str() {
            "g" => Clef {
                line_middle_c: 10,
                name: "Treble".to_string(),
            },
            "c" => match line {
                Some(1) => Clef {
                    line_middle_c: 8,
                    name: "Soprano".to_string(),
                },
                Some(2) => Clef {
                    line_middle_c: 6,
                    name: "Mezzo Soprano".to_string(),
                },
                Some(3) => Clef {
                    line_middle_c: 4,
                    name: "Alto".to_string(),
                },
                Some(4) => Clef {
                    line_middle_c: 2,
                    name: "Tenor".to_string(),
                },
                Some(5) => Clef {
                    line_middle_c: 0,
                    name: "Baritone".to_string(),
                },
                _ => panic!("A C-type clef needs a line."),
            },
            "f" => Clef {
                line_middle_c: -2,
                name: "Bass".to_string(),
            },
            "percussion" => Clef {
                line_middle_c: 4,
                name: "Percussion".to_string(),
            },
            _ => panic!("{} not a clef sign", sign),
        }
    }

    pub fn line_index_at_pitch(&self, pitch: &Pitch) -> i32 {
        self.line_middle_c + (3 - pitch.octave) * 7 + (7 - pitch.step.steps_from_c)
    }
}

impl Default for Clef {
    fn default() -> Self {
        Clef::parse("g", None)
    }
}
