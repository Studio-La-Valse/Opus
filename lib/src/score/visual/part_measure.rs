use crate::drawable::elements::line::Line;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::core::voice::Voice;
use crate::score::layout_options::APP_DEFAULTS;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::chord::Chord;
use crate::score::visual::layoutable::LayoutParams;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct PartMeasure {
    pub part_id: String,
    pub number: u32,

    pub specified_width: Option<f32>,
    pub final_width: f32,

    pub width: f32,
    pub height: f32,
    pub origin: XY,

    pub chords: BTreeMap<Voice, Vec<Chord>>,
    pub ledgers: Vec<Line>,

    pub color: Color,

    pub ledger_thickness: f32,
    pub ledger_width: f32,
}

impl PartMeasure {
    pub fn new(part_id: String, number: u32) -> Self {
        Self {
            part_id,
            number,
            ..Default::default()
        }
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for grace in [false, true] {
            let chord_groups = collect_voices(&mut self.chords, grace);

            for mut chords in chord_groups {
                strategy.rebeam(&mut chords)
            }
        }
    }
}

impl PartMeasure {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            score_defaults,
            user_layout,
            font,
        } = params;

        self.color = params.foreground_color();

        self.ledger_thickness = user_layout
            .staff
            .line_width
            .or(score_defaults.appearance.staff)
            .or(font.layout.staff.line_width)
            .unwrap_or(APP_DEFAULTS.staff.line_width);

        for chord in self.chords.values_mut().flatten() {
            chord.resolve_layout(params);
        }
    }
}

fn collect_voices(
    chord_groups: &mut BTreeMap<Voice, Vec<Chord>>,
    grace: bool,
) -> Vec<Vec<&mut Chord>> {
    let mut result: Vec<Vec<&mut Chord>> = Vec::new();

    for chords in chord_groups.values_mut() {
        let mut group: Vec<&mut Chord> = Vec::new();

        for chord in chords {
            if chord.grace != grace {
                continue;
            }

            group.push(chord);
        }

        result.push(group);
    }

    result
}
