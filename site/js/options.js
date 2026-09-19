// The option manifest for the render page's pane: one descriptor per knob the
// <music-xml> element exposes, mirroring lib/src/score/user_layout.rs field
// for field - with three exceptions, each noted at its own entry below:
// titleFont/lyricFont/groupNameFont (UserLayout is Copy, so it excludes font
// families) and pageOrientation (page arrangement is a component-level, CSS
// concern now that the engine no longer arranges pages relative to each other
// at all - there is no UserLayout field for it; the gap between pages is a
// fixed 10px in the component and is not exposed here at all). Adding a
// UserLayout knob here is the only change needed to add it to the pane -
// render.js builds the whole thing by iterating this array, the same
// data-driven intent UserLayout's own doc comment asks for ("Don't mirror it
// into a parallel options struct").
//
// Defaults are copied from `impl Default for AppDefaults`
// (lib/src/score/app_defaults.rs) - they're also what each control shows as
// its placeholder/rest value, since a control left at its default writes
// nothing (see render.js).
//
// `kind`:
//   "color"  - <input type=color> + an alpha slider, composing #RRGGBBAA.
//   "number" - a linked range slider + number box. `unit` is display-only.
//   "enum"   - a <select> over `values`.
//   "font"   - free-text CSS font-family; blank means "unset".
//   "boolean-attribute" - a checkbox toggling a real HTML attribute (`debug`
//              is the only one; every other knob is a CSS custom property).
export const OPTION_GROUPS = [
  "Page",
  "Staff & barlines",
  "Measure starts",
  "Beams & stems",
  "Note sizes & dots",
  "Ties",
  "Group symbols",
  "Group names",
  "Fonts",
  "Debug",
];

const GROUP_SYMBOL_VALUES = ["none", "brace", "bracket", "line", "square"];

export const OPTIONS = [
  // --- Page ---------------------------------------------------------------
  {
    group: "Page",
    name: "pageColor",
    kind: "color",
    default: "#ffffff",
    label: "Page colour",
    help: "Background the page is painted with.",
  },
  {
    group: "Page",
    name: "foregroundColor",
    kind: "color",
    default: "#000000",
    label: "Foreground colour",
    help: "Colour used for staves, notation and text.",
  },
  {
    group: "Page",
    // Component-level, not a UserLayout field: music-xml.js's own
    // _applyPageOrientation reads --page-orientation to flex-direction the
    // stacked page canvases, the CSS layout problem the engine used to solve
    // by arranging pages relative to each other.
    name: "pageOrientation",
    kind: "enum",
    values: ["vertical", "horizontal"],
    // The site's own default (site/css/style.css sets --page-orientation:
    // vertical), not "unset" (the component's own default, absent the custom
    // property, is also column/vertical - see music-xml.js) - so this has to
    // match that rule for "at its default writes nothing" to stay true here.
    default: "vertical",
    label: "Page orientation",
    help: "Portrait (vertical) or landscape (horizontal) pages.",
  },

  // --- Staff & barlines -----------------------------------------------------
  {
    group: "Staff & barlines",
    name: "staffLineWidth",
    kind: "number",
    default: 1.1,
    min: 0,
    max: 6,
    step: 0.1,
    unit: "tenths",
    label: "Staff line width",
    help: "Thickness of the five staff lines.",
  },
  {
    group: "Staff & barlines",
    name: "lightBarline",
    kind: "number",
    default: 1.875,
    min: 0,
    max: 10,
    step: 0.125,
    unit: "tenths",
    label: "Light barline thickness",
    help: "Thickness of an ordinary barline.",
  },
  {
    group: "Staff & barlines",
    name: "heavyBarline",
    kind: "number",
    default: 5,
    min: 0,
    max: 20,
    step: 0.5,
    unit: "tenths",
    label: "Heavy barline thickness",
    help: "Thickness of a final/heavy barline stroke.",
  },

  // --- Measure starts ---------------------------------------------------------
  {
    group: "Measure starts",
    name: "measureStartClefPadding",
    kind: "number",
    default: 5,
    min: 0,
    max: 40,
    step: 0.5,
    unit: "tenths",
    label: "Clef padding",
    help: "Gap between the measure's left edge and the opening clef.",
  },
  {
    group: "Measure starts",
    name: "measureStartKeySignaturePadding",
    kind: "number",
    default: 5,
    min: 0,
    max: 40,
    step: 0.5,
    unit: "tenths",
    label: "Key signature padding",
    help: "Gap between the opening clef column and the key signature.",
  },
  {
    group: "Measure starts",
    name: "measureStartTimeSignaturePadding",
    kind: "number",
    default: 5,
    min: 0,
    max: 40,
    step: 0.5,
    unit: "tenths",
    label: "Time signature padding",
    help: "Gap between the key signature column and the opening time signature.",
  },

  // --- Beams & stems --------------------------------------------------------
  {
    group: "Beams & stems",
    name: "beamThickness",
    kind: "number",
    default: 5,
    min: 0,
    max: 20,
    step: 0.5,
    unit: "tenths",
    label: "Beam thickness",
    help: "Thickness of a single beam stroke.",
  },
  {
    group: "Beams & stems",
    name: "beamSpacing",
    kind: "number",
    default: 1.5,
    min: 0,
    max: 10,
    step: 0.5,
    unit: "tenths",
    label: "Beam spacing",
    help: "Gap between adjacent beam levels.",
  },
  {
    group: "Beams & stems",
    name: "stemThickness",
    kind: "number",
    default: 1,
    min: 0,
    max: 5,
    step: 0.1,
    unit: "tenths",
    label: "Stem thickness",
    help: "Thickness of a note stem.",
  },

  // --- Note sizes & dots ------------------------------------------------------
  {
    group: "Note sizes & dots",
    name: "noteSizeGrace",
    kind: "number",
    default: 0.66,
    min: 0.1,
    max: 1.5,
    step: 0.01,
    unit: "×",
    label: "Grace note size",
    help: "Fraction of full size a grace note (and everything it owns - notehead, stem, flag, beams) is drawn at.",
  },
  {
    group: "Note sizes & dots",
    name: "noteSizeCue",
    kind: "number",
    default: 0.66,
    min: 0.1,
    max: 1.5,
    step: 0.01,
    unit: "×",
    label: "Cue note size",
    help: "Fraction of full size a cue note is drawn at.",
  },
  {
    group: "Note sizes & dots",
    name: "dotRadius",
    kind: "number",
    default: 2,
    min: 0,
    max: 10,
    step: 0.5,
    unit: "tenths",
    label: "Augmentation dot radius",
    help: "Radius of an augmentation dot.",
  },
  {
    group: "Note sizes & dots",
    name: "dotSpacing",
    kind: "number",
    default: 5,
    min: 0,
    max: 20,
    step: 0.5,
    unit: "tenths",
    label: "Augmentation dot spacing",
    help: "Gap from the notehead/rest to the first dot, and between successive dots.",
  },

  // --- Ties -----------------------------------------------------------------
  {
    group: "Ties",
    name: "tieEndpointThickness",
    kind: "number",
    default: 1,
    min: 0,
    max: 10,
    step: 0.1,
    unit: "tenths",
    label: "Tie endpoint thickness",
    help: "Tie thickness where it meets a notehead.",
  },
  {
    group: "Ties",
    name: "tieMidpointThickness",
    kind: "number",
    default: 2.2,
    min: 0,
    max: 10,
    step: 0.1,
    unit: "tenths",
    label: "Tie midpoint thickness",
    help: "Tie thickness at its widest point.",
  },
  {
    group: "Ties",
    name: "tieHeightRatio",
    kind: "number",
    default: 0.15,
    min: 0,
    max: 1,
    step: 0.01,
    unit: "×",
    label: "Tie height ratio",
    help: "A tie's arc height as a fraction of its horizontal span, before clamping.",
  },
  {
    group: "Ties",
    name: "tieHeightMin",
    kind: "number",
    default: 5,
    min: 0,
    max: 60,
    step: 1,
    unit: "tenths",
    label: "Tie height (min)",
    help: "Lower clamp on a tie's arc height.",
  },
  {
    group: "Ties",
    name: "tieHeightMax",
    kind: "number",
    default: 16,
    min: 0,
    max: 60,
    step: 1,
    unit: "tenths",
    label: "Tie height (max)",
    help: "Upper clamp on a tie's arc height.",
  },
  {
    group: "Ties",
    name: "tieNoteGap",
    kind: "number",
    default: 2,
    min: 0,
    max: 20,
    step: 0.5,
    unit: "tenths",
    label: "Tie note gap",
    help: "Gap between a notehead's edge and the tie tip that meets it.",
  },
  {
    group: "Ties",
    name: "tieVerticalOffset",
    kind: "number",
    default: 5,
    min: 0,
    max: 20,
    step: 0.5,
    unit: "tenths",
    label: "Tie vertical offset",
    help: "Offset from a notehead's vertical centre to the tie tip.",
  },
  {
    group: "Ties",
    name: "tieBreakInset",
    kind: "number",
    default: 10,
    min: 0,
    max: 40,
    step: 1,
    unit: "tenths",
    label: "Tie break inset",
    help: "Margin between the barline and the far end of the opening fragment of a tie broken across a system break.",
  },
  {
    group: "Ties",
    name: "tieBreakFragment",
    kind: "number",
    default: 20,
    min: 0,
    max: 60,
    step: 1,
    unit: "tenths",
    label: "Tie break fragment length",
    help: "Length of the closing fragment of a broken tie (and the minimum length of the opening one).",
  },

  // --- Group symbols ----------------------------------------------------------
  {
    group: "Group symbols",
    name: "sectionSymbol",
    kind: "enum",
    values: GROUP_SYMBOL_VALUES,
    default: "bracket",
    label: "Section symbol",
    help: "Symbol binding a section's part-groups together, overriding the document.",
  },
  {
    group: "Group symbols",
    name: "partGroupSymbol",
    kind: "enum",
    values: GROUP_SYMBOL_VALUES,
    default: "brace",
    label: "Part-group symbol",
    help: "Symbol binding a part-group's parts together, overriding the document.",
  },
  {
    group: "Group symbols",
    name: "partSymbol",
    kind: "enum",
    values: GROUP_SYMBOL_VALUES,
    default: "brace",
    label: "Part symbol",
    help: "Symbol binding one part's own staves together.",
  },
  {
    group: "Group symbols",
    name: "sectionSymbolGap",
    kind: "number",
    default: 5,
    min: 0,
    max: 40,
    step: 1,
    unit: "tenths",
    label: "Section symbol gap",
    help: "Distance from the system's left edge to a section symbol's stroke.",
  },
  {
    group: "Group symbols",
    name: "partGroupSymbolGap",
    kind: "number",
    default: 2,
    min: 0,
    max: 40,
    step: 1,
    unit: "tenths",
    label: "Part-group symbol gap",
    help: "Padding between a section's symbol and a part-group's symbol inside it.",
  },
  {
    group: "Group symbols",
    name: "partSymbolGap",
    kind: "number",
    default: 5,
    min: 0,
    max: 40,
    step: 1,
    unit: "tenths",
    label: "Part symbol gap",
    help: "Padding between a part-group's symbol and a part's own symbol inside it.",
  },
  {
    group: "Group symbols",
    name: "groupBracketThickness",
    kind: "number",
    default: 5,
    min: 0,
    max: 20,
    step: 0.5,
    unit: "tenths",
    label: "Bracket thickness",
    help: "Thickness of a bracket symbol's vertical stroke.",
  },
  {
    group: "Group symbols",
    name: "groupLineThickness",
    kind: "number",
    default: 1.6,
    min: 0,
    max: 10,
    step: 0.1,
    unit: "tenths",
    label: "Line symbol thickness",
    help: "Thickness of a line symbol.",
  },
  {
    group: "Group symbols",
    name: "groupSquareThickness",
    kind: "number",
    default: 1.2,
    min: 0,
    max: 10,
    step: 0.1,
    unit: "tenths",
    label: "Square symbol thickness",
    help: "Thickness of a square symbol's stroke, spine and arms.",
  },
  {
    group: "Group symbols",
    name: "groupSquareArm",
    kind: "number",
    default: 8,
    min: 0,
    max: 40,
    step: 1,
    unit: "tenths",
    label: "Square symbol arm length",
    help: "How far a square symbol's arms reach toward the system.",
  },

  // --- Group names ------------------------------------------------------------
  {
    group: "Group names",
    name: "groupNameSize",
    kind: "number",
    default: 16,
    min: 0,
    max: 60,
    step: 1,
    unit: "tenths",
    label: "Group name font size",
    help: "Font size for a part/part-group name.",
  },
  {
    group: "Group names",
    name: "groupNamePadding",
    kind: "number",
    default: 10,
    min: 0,
    max: 40,
    step: 1,
    unit: "tenths",
    label: "Group name padding",
    help: "Padding between a name's right edge and the symbol it sits beside.",
  },

  // --- Fonts --------------------------------------------------------------
  // Not UserLayout fields (it's Copy) - these three are separate CSS custom
  // properties the component reads directly. See music-xml.js's _render().
  {
    group: "Fonts",
    name: "titleFont",
    kind: "font",
    default: "serif",
    label: "Title font",
    help: "Font family for title/work-level text.",
  },
  {
    group: "Fonts",
    name: "lyricFont",
    kind: "font",
    default: "serif",
    label: "Lyric font",
    help: "Font family for lyrics.",
  },
  {
    group: "Fonts",
    name: "groupNameFont",
    kind: "font",
    default: "serif",
    label: "Group name font",
    help: "Font family for part/part-group names.",
  },

  // --- Debug ----------------------------------------------------------------
  {
    group: "Debug",
    name: "debug",
    kind: "boolean-attribute",
    default: false,
    label: "Debug overlay",
    help: "Draws internal layout bounds over the score.",
  },
];

// "pageColor" -> "page-color", matching music-xml.js's own cssPropertyFor -
// the CSS custom property a "number"/"color"/"enum" option is read from.
export function cssPropertyFor(name) {
  return name.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`);
}
