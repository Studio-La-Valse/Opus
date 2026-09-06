# Roadmap

Work that is known, wanted, and not done. Ordered: the first section is what gets
picked up next, the rest is a holding list rather than a queue.

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

---

## Also outstanding

Smaller, independent, and in no particular order.

| Item | Where |
|---|---|
| Staves are always five lines. `<staff-lines>` is not read, and `render_staff` loops `0..5`. | `base_renderer.rs`, `Staff::LINES` |
| Tablature is a clef glyph and nothing else — no six-line staff, no fret numbers, no `<staff-tuning>`. Notes on a tab staff are placed by pitch. | `Clef::Tab` |
| A grace or cue note's accidental is drawn full size. `Accidental` never carries a `NoteScale` — its `scale` is fixed at 1 on construction, so it misses both the `<note-size>` reduction and the staff's own content scaling. | `accidental.rs` |
| Beam thickness follows a grace group but not a cue one. `arrange_beams` reduces by `note_size_grace` when `Chord::grace`, and a chord knows nothing else; giving `Chord` a `NoteKind` would also split beam groups by kind, which is the part worth thinking about first. | `part_measure.rs::arrange_beams` |
| Only the first `<key>` in an `<attributes>` is read, so per-staff key signatures (`<key number="n">`) are ignored. | `walker.rs` |
| The baritone clef's fifth sharp (A♯) is placed below the bottom staff line; every other clef keeps its key signature on the staff. Reachable at five sharps. | `Clef::sharp_lines` |
| A `Text` element contributes only its anchor point to `compute_bounds`, so a glyph at the edge of a score is not counted in the bounds the wasm canvas and SVG view box are sized from. | `drawable_element.rs::accumulate_bounds` |
| `LAYOUT_OPTIONS` in the web component is a hand-maintained list naming six of `UserLayout`'s fields; the tie, dot and beam knobs cannot be reached from CSS. | `web/music-xml.js` |
| Curves are sampled into polygons because there is no path primitive. A real `DrawableElement::Path` would serve ties, slurs, hairpins and ottavas, and the PDF sink already has `cubic_to`. The full migration is written out in `tie_arc`'s doc comment. | `tie.rs` |
