#[derive(Debug)]
pub struct Step {
    pub steps_from_c: i32,
    pub alter: i32,
}

impl Step {
    pub fn parse(string: &str, alter: i32) -> Self {
        match string.to_lowercase().as_str() {
            "c" => Step {steps_from_c: 0, alter, },
            "d" => Step {steps_from_c: 1, alter, },
            "e" => Step {steps_from_c: 2, alter, },
            "f" => Step {steps_from_c: 3, alter, },
            "g" => Step {steps_from_c: 4, alter, },
            "a" => Step {steps_from_c: 5, alter, },
            "b" => Step {steps_from_c: 6, alter, },
            _ => panic!("{} not a valid step", string)
        }
    }
}