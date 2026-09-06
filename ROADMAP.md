# Roadmap

Work that is known, wanted, and not done. Ordered: the first section is a major (multi commit-) feature that that needs an implementation as soon as possible. 

---

## 1. Beaming

The largest outstanding gap, and the one to do next.

### 1a. Beam groups that outlive a measure

A beam group cannot currently leave the measure it starts in.
`PartMeasure::rebeam` and `PartMeasure::arrange_beams` both begin at
`collect_voices(&mut self.chords, grace)`, which is *this* measure's chords, so
`create_beam_groups` never sees a chord from anywhere else. The drawn beams are
`PartMeasure::beams`, a `Vec<Polygon>` the measure owns.

That rules out three things that occur in real music:

- **Cross-measure** beams — the common case, and the one worth having first.
- **Cross-system** beams, where the group is broken by a system break and each
  half is drawn against its own system.
- **Cross-page** beams, which are the same case as cross-system: the two systems
  simply sit on different pages.

**Ties already solved this exact shape, and the pattern should be reused rather
than reinvented.** `Score::ties` is a flat list of `NoteId` pairs owned by the
score rather than by any node in the page tree, because the two endpoints may be
measures, systems or pages apart. `arrange_ties` runs last, after
`LayoutEngine::arrange_pages`, when every note finally has an absolute position,
and files the resulting arcs under `System::ties` keyed by `(page, system)`. The
cross-page case needed no code of its own once the fragments were keyed that way.
`lib/src/score/visual/tie.rs` and `tie_arranger.rs` carry the full reasoning, and
`split_tie` is deliberately a pure function over anchors and extents so it can be
tested without a multi-system fixture.

A beam group is the same kind of object: a relation between chords that the
containment hierarchy cannot express. The shape to aim for is a flat
`Score::beam_groups`, resolved after arrange into per-system geometry.

What beams need that ties did not:

- A group is *n* chords, not two endpoints, and the beam's slant is fitted across
  all of them (`create_ray`), so the split has to decide the slant per fragment.
- Stem lengths are adjusted to meet the beam (`adjust_stem_lengths`), which means
  the resolution pass has to write back into the chords, not just emit geometry.
  Ties only ever read.
- Secondary beam levels can start and stop independently of level 1, so a split
  has to be applied per level.

### 1b. A rebeam strategy worth the name

`SimpleRebeamStrategy` assigns `Start` to the first stem, `End` to the last and
`Continue` to everything between, at every beam level the note's duration
warrants. Three things it gets wrong:

- **No hooks.** A dotted-eighth/sixteenth pair should give the sixteenth a
  backward hook; the current strategy gives it a full secondary beam back to the
  dotted eighth. `BeamType::HookStart` / `HookEnd` exist and are drawn correctly
  — nothing ever infers them.
- **No beat grouping.** A beam group should break at beat boundaries: 6/8 groups
  in threes, 4/4 in twos. The strategy has no notion of the time signature and
  beams whatever it is handed as one run.
- **The trigger is too narrow.** `OnlyWhenRequiredRebeamStrategy` defers to
  `requires_rebeam`, which compares how many beams a note *declares* against how
  many its duration *warrants*. A group whose counts are all correct but whose
  types do not form a `begin…end` run passes straight through untouched — that is
  exactly the malformation `BeamGroupVisitor` now warns about and
  `beam_level_ends_at` infers around. A strategy that validated the run's shape
  would repair those instead of leaving the renderer to guess.

### 1c. Potentially free/cheap fixes for these currently known issues: 
- Include: Beam slant is clamped to a flat `MAX_BEAM_SLANT_DY` of two line spaces. Two spaces is indeed the conventional maximum, but a clamp is all it is: the slant a group actually wants follows the interval it spans, how many notes it has and where they sit on the staff. 
- A beam hook is a fixed `HOOK_LENGTH` of 7.5 tenths. It should be inferred from the space available between the two stems and clamped to a maximum. 

---

## Simple fixes and features: short term goals

Smaller, independent, tentatively ordered by consecutive feature impact.

- support bar lines for a single line staff: should extent one staff space on top and below of the single line (so 20 tenths in total). 

- Add support for a user to define layout from a deserialized file. Decide yml, ini, json, etc
Keep current arguments to cli: allow them as additional overrides on top of the user layout. Review structure of user layout particularly, should be flat list or nested? Also fully mirror app defaults and user layout: no option should exit in one that doesnt exist in the other - however user layout options are always optional, app defaults are static and readonly, always have a value assigned. User layout file should always be optional, not required for cli functionality.

- Add support for different brace styles. Now only curly drawing, starting with rectangluar. Read the type from the music xml document. Should be flexible in size as well, like the bracket spine is. Also add support for brace alternatives as defined in smufl. If feasable, add support for user defined style: brace vs square for partgroups, etc. 

- Evaluate/review: a text element should be a text box so that it has a measurable bounding box. Should not change visible behavior at all. We could provide support for box background color as well. 

- Partgroups and part names should be drawn left of the (part(-group)) brace or section bracket if it exists. Should add a new font type, now only music and titles; should add part names font type to support a user defined font (as well as app default). Include in this feature: support for user defined spacing of bracket and brace left of systems. Sections should not have a (potential) name attached.

- One `ELEMENT_PADDING` constant spaces the clef, the key signature and the time signature alike. It should be split into a padding value per element, each reachable through both `AppDefaults` and `UserLayout`. 

- Scale grace and cue note's accidental, flag and beam group. Decide how to handle vecs of chord with mixed grace, cue and normal notes: validation is a must.

- Review: should lib be split into separate crates, akin .net project setup? Review internal dependency tree. For example: Review pdf export dependencies in lib, should only live in cli? Maybe not required/conventional for Rust projects.

- Ties and slurs are now multi segment polygons because of thickness. Update path support - or explicitly decide not to.

- Review staff line indexing: ours start at 0 from top to bottom, musicxml counts from 1 bottom to top. Clef anchor lines work on 5 line staves but break on anything else. Either a conversion method or review our internal drawing engine. 

- Potential bug: a part whose first measure carries no `<print>` is engraved in a different system than one that does. `system.index` is per-part state, reset per part and incremented only inside `enter_print`, where measure 1 implies a new system — so a part without the element stays at index 0 while its neighbours move to 1. Real exports write `<print>` in every part, which is what keeps this latent; a validation rule should report a measure where the parts disagree about carrying one. 

- Generalize smufl parsing from some point onwards: so far managable in code but defining thousands of smufl glyphs in code is undesirable.

- Generalize bounding boxes, every score element should have an inherent bounding box which is always drawn when --debug is provided.

- Access Smufl data from installed system wide files (as documented by smufl itself), optionally providing currently supported smufl (meta-)data through cli args. 

- Support for other smufl fonts.

- Support for lyrics. 

- Support for every articulation imaginable. 

- Support for slurs. 


## Long term goals: 
- Installer, create publication, system wide available `opus` cli. Release once cli reaches stable point, do not wait for full desktop/mobile/web support. Prioritize cli.

- Publication of visual score, smufl and musicxml crates, so users may embed in third party applications.

- Full tablature support. 

- Zero panic render pipeline. Should be able to handle every imaginable musicxml document, guaranteed to not panic once passes validation. Problems in document should be skipped and displayed if --debug is provided (such as a red border around a measure that has content the cli was not able to parse.)

- Support for different layout engines. I image for example a layout engine where we can override page size and measures and systems restructure without reading the musicxml print attributes. Support for single page scroll in scope. Current layout engine must keep support. 

- Support for mxl (zipped musicxml document). Should be able to export to mxl as well as a separate cli command `export`. 

- Support for older musicxml document versions. Should be able to export to any given format as a separate cli command `export`.

- Support for finale, sibelius and musescore interop. 

- Auto fix document issues. 

- Improved Web/WASM support.

- Desktop app support - see /platforms skeleton.

- Mobile app support - see /platforms skeleton.