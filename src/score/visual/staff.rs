#[derive(Default)]
pub struct Staff {
    pub hidden: bool,
    pub distance_specified: Option<f32>,
}

impl Staff {
    // 5 lines, 4 spaces, 10 tenths for each space according to MusicXML spec.
    pub const SIZE: i32 = 40;
}
