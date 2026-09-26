# Roadmap

Work that is known, wanted, and not done, grouped by kind — bugs first, then
small fixes, features, open questions, and the long term.

---

## 1. Bugs

Defects in what is already built, not missing features.

- **A part whose first measure carries no `<print>` is engraved in a different
  system than one that does.** `system.index` is per-part state, reset per part
  and incremented only inside `enter_print`, where measure 1 implies a new
  system — so a part without the element stays at index 0 while its neighbours
  move to 1. Real exports write `<print>` in every part, which is what keeps this
  latent; a validation rule should report a measure where the parts disagree
  about carrying one.

- **Tie direction assumes a five-line staff.** `MIDDLE_STAFF_LINE` in `tie.rs` is
  a hardcoded 4, so on any staff that is not five lines the over/under inference
  is measured against the wrong line. Staves now carry their declared line count
  and `Clef::anchor_line` already takes it, which makes this the remaining
  five-line assumption in the render path. See also the indexing question in
  section 4.
- **A section measures itself from its first part-group, not its first visible
  one.** `Section::first_visible_staff_distance` reads
  `part_groups.values().next()` unconditionally, while `PartGroup`'s version of
  the same method skips hidden parts. A section whose first part-group is
  entirely hidden therefore places its measures, its barline and its group
  symbol against the wrong distance.

- **Notes without pitch or default-x value are currently skipped altogether.** These
  notes have a valid musical purpose. Notes without default-x are difficult to handle
  without a custom layout engine, so validate and produce warnings when no default-x 
  attribute for a note is supplied - then we may skip in content visitor after all. 
  Notes without pitch (purcussive notes) should in fact be handled like any other.

- **The browser path cannot read UTF-16 MusicXML.** `web/music-xml.js`'s
  `_loadFile` does `await response.text()`, which decodes as UTF-8;
  `assets/xmlsamples/MozaChloSample.musicxml` is UTF-16BE and
  `MozaVeilSample.musicxml` is UTF-16LE, so both fail to parse in the browser
  — they render fine through the CLI, which sniffs the BOM in `read_musicxml`
  (`cli/src/commands/mod.rs`) — and are excluded from the exhibition site's
  sample set (`scripts/build-site.sh`) for the same reason. Affects any
  consumer feeding the component a Finale export, not just the site. Fix is
  roughly `arrayBuffer()` + BOM sniff + `TextDecoder`, ~15 lines.

---

## 2. Fixes

Small and well-specified. Each is a single change and none depend on each other.

- Scale grace and cue notes' accidental, flag and beam group. Decide how to
  handle vecs of chords with mixed grace, cue and normal notes: validation is a must.

- Resolve the layout once, into a `ClosedLayout`. Every option is
  resolved at its point of use by a hand-written chain —
  `user.or(document).or(font).unwrap_or(APP_DEFAULTS)` — about fifteen of them,
  and adding the font tier meant editing each. A site that skips a tier still
  compiles. Every tier but the last is already a partial layout: the user's and
  the font's are `UserLayout`s, and the document's `Appearance` needs only a
  `UserLayout::from_appearance`. So `layout_options!` could generate a `close()`
  that fills the gaps from `APP_DEFAULTS` into a struct with no `Option`s,
  resolved once per `arrange_score` with `overlay`. `LayoutParams` would shrink
  to that plus the font.

  Group symbols don't close cleanly: the document declares one per
  `<part-group>`, and a user override has to beat each declaration, so the
  closed layout keeps the user's symbol as an `Option` beside the fallback. Tie
  height sanitizing (`ordered_bounds`) would move into `close()`. The refactor
  should change no output.

- Quantize beams onto the staff lines. `fit_beam` decides a beam's slant and
  height from the notes and stems alone, so a beam end can land anywhere
  relative to a staff line, including a thin wedge of white between beam and
  line. Engraving convention snaps each end inside the staff to sit on,
  straddle or hang from a line; beams entirely outside the staff are exempt.
  Applied after the slant and push-out, moving the line outward only, so no
  stem gets shorter than its natural length.

## 3. Features

Larger than a fix. Several will want a code-anchored breakdown before they
start. Tentatively ordered by consecutive feature impact.

- **A rebeam strategy worth the name.** `SimpleRebeamStrategy` assigns `Start`
  to the first stem, `End` to the last and `Continue` to everything between, at
  every beam level the note's duration warrants. Three things it gets wrong:

  - **No hooks.** A dotted-eighth/sixteenth pair should give the sixteenth a
    backward hook; the current strategy gives it a full secondary beam back to
    the dotted eighth. `BeamType::HookStart` / `HookEnd` exist and are drawn
    correctly — nothing ever infers them.
  - **No beat grouping.** A beam group should break at beat boundaries: 6/8
    groups in threes, 4/4 in twos. The strategy has no notion of the time
    signature and beams whatever it is handed as one run.
  - **The trigger is too narrow.** `OnlyWhenRequiredRebeamStrategy` defers to
    `requires_rebeam`, which compares how many beams a note *declares* against
    how many its duration *warrants*. A group whose counts are all correct but
    whose types do not form a `begin…end` run passes straight through untouched
    — that is exactly the malformation `BeamGroupVisitor` warns about and
    `beam_level_ends_at` infers around. A strategy that validated the run's
    shape would repair those instead of leaving the renderer to guess.

- **A layout file for `<music-xml>`.** The CLI layers `--layout <file.toml>`
  under its flags with `UserLayout::overlay`; the web component has only its
  CSS custom properties. A `layout` attribute naming a file to fetch, with the
  custom properties overlaid on top, would give it the same two tiers. Nothing
  on the wasm side needs to change: `RenderOptions.layout` already takes one
  merged `UserLayout`, so the merge belongs in JS (or in a small wasm helper
  exposing `overlay`), and the file would be JSON or TOML parsed in the browser.

- **The rest of what a `<part-group>` says.** `<group-symbol default-x>` is the
  document declaring how far from the system its symbol sits, and every real
  sample carries one — ActorPrelude writes `-5` for its brackets and `-9` for
  its braces, MozartTrio `-10`. Those are close enough to the gaps `AppDefaults`
  now holds that nothing looks wrong, which is the only reason ignoring them has
  been tolerable. `<group-barline>` is not read at all. And
  `<attributes><part-symbol>` is the per-part group symbol: the `declared` slot
  on a part's `GroupSymbol` exists for it and is always `None` until something
  fills it. No sample in `assets/` uses one.

- **Text fonts from the document.** `title.font` / `lyric.font` /
  `group_name.font` resolve user → the SMuFL font's `textFontFamily` → `serif`,
  with no document tier between the first two. MusicXML declares one in
  `<defaults>`: `<lyric-font>` for lyrics and `<word-font>` for other text, both
  with a `font-family` that is itself a comma-separated list. Most samples
  write `Times New Roman`. `<lyric-font>` maps onto `lyric.font` directly.
  `<word-font>` is the default for words generally, so whether it covers titles
  (which MusicXML usually carries as `<credit>` with its own fonts) and part
  names needs deciding. Its `font-size` is a second option nothing reads yet.

- **Generalize SMuFL parsing** from some point onwards: so far manageable in
  code, but defining thousands of SMuFL glyphs in code is undesirable.

- **Generalize bounding boxes**: every score element should have an inherent
  bounding box, always drawn when `--debug` is provided.

- **Web formats for Leland and Finale Maestro.** Both ship only OTF, so the
  site serves them as OTF (about 0.3 MB and 0.6 MB, loaded the first time
  each is chosen) where Bravura gets woff2. Converting them once and committing
  the woff2 files would roughly halve that.

- **`<music-font font-size>`.** The walk reads the family list and ignores the
  size, which every sample writes. It should decide what one staff space is,
  and so conflicts with the document's `<scaling>`; which one wins needs
  deciding.

- **`opus font install` beyond a local folder.** It installs a package already
  on disk. Downloading upstream releases, and being called from package
  installers (brew, MSI, deb) so `opus` can ship with its fonts, belongs with
  the packaging work. On Windows it copies OTFs to the per-user font folder
  without the registry entry Windows itself adds, so `opus` finds them but
  other applications may not.

- **Tuplets.** The number, the bracket and its hooks, nested tuplets, and
  `<time-modification>` feeding the duration maths. A tuplet bracket spans the
  same run of chords a beam group does, and when that run is beamed the bracket
  is conventionally suppressed in favour of the bare number — so a tuplet wants
  the same shape: a run rebuilt from document order by a whole-score pass after
  arrange, the way `BeamArranger::collect_runs` rebuilds beam groups, split per
  system where it crosses a break.

- **Barline types, including repeats.** Light and heavy, double and final,
  repeat dots and their forward/backward direction, volta brackets for endings,
  and the segno / coda / D.C. / D.S. apparatus that goes with them. Repeat
  barlines interact with system breaks, so this wants the layout side settled
  first.

- **Dynamics.** The SMuFL dynamic glyphs and their placement below (or above)
  the staff, per voice, with the vertical space they claim reserved rather than
  overlapped.

- **Notehead shape alternatives.** `<notehead>`: x, diamond, slash, triangle and
  the rest. Needed before percussion is readable, and the shape has to reach the
  glyph lookup rather than being decided by duration alone.

- **Lyrics.** The font is already plumbed end to end — `RenderFonts::lyric`,
  the `lyric.font` layout option — so what is missing is
  reading `<lyric>`, placing the syllables, and the vertical space they claim.

- **Every articulation imaginable.**

- **Slurs.** `tie.rs` was written as the seam: the arc geometry is free functions
  over anchors, and a slur is the same arc between different anchors. What a slur
  adds is that its endpoints are not pinned down by pitch the way a tie's are, so
  choosing the anchors — and the collision avoidance that follows from it — is
  the actual work.

---

## 4. Reviews and decisions

Open questions. Each wants an answer written down, and the answer may be "no".

- **Should `lib` be split into separate crates**, akin to a .NET project setup?
  Review the internal dependency tree. PDF export is the concrete case: `lib`
  pulls `pdf-writer` and `ttf-parser` for `drawable::canvas::pdf`, which only the
  cli consumes today. It should not simply move into `cli`, though — the design
  note at the end of `wasm/src/lib.rs` plans a `render_pdf` on the wasm `Score`
  built on that same canvas and `write_pdf`. A separate crate that both depend on
  is the shape that serves both. `lib` also owns the svg and flat-buffer canvases
  as a matched set, so the real question is where the whole `drawable::canvas`
  layer belongs. May not be required or conventional for Rust projects.

- **Path support for ties.** Ties are now multi-segment polygons because of
  thickness. Update path support — or explicitly decide not to. (Slurs are not
  built yet; they are in section 3.)

- **Staff line indexing.** Ours start at 0 from top to bottom, MusicXML counts
  from 1 bottom to top. Either a conversion method, or review our internal
  drawing engine. `Clef::anchor_line` now takes the staff's line count and
  centres the unpitched clefs, so the clef case is handled; what remains is the
  convention itself and the assumptions still riding on it — see the tie
  direction bug in section 1.

- **Should a declared `<group-symbol>` beat the structural guard?** A group
  symbol is drawn only when there is more than one thing for it to bind: more
  than one part-group in a section, more than one part and more than one visible
  staff in a part-group. So a document that explicitly writes
  `<group-symbol>bracket</group-symbol>` around a single part gets nothing,
  though MusicXML would allow it. The guard is what stops a one-part score
  sprouting a bracket, since `locate_or_create_part` builds a section and a
  part-group for every part whether the document named one or not — so removing
  it outright is not the answer. Letting an explicit declaration override it
  might be.

- **Ties should only tie to next immediate note**. Should a tied note search for
  any note with the same NoteId, potentially skipping notes, or only tie if the
  following note has the same NoteId? If the next note does not have the same 
  NoteId, what is the behavior of the tie? Can we simplify code in this case?

- **Should the user layout pass and the measure/arrange pass be separate?**
  Now, they are in the same function call, which greatly improves code readability
  (no separate score tree builder required) - but now score may unnecessarily 
  be remeasured and arranged, for example on color change. 

---

## 5. Long term goals

- Installer, create publication, system wide available `opus` cli. Release once
  the cli reaches a stable point, do not wait for full desktop/mobile/web
  support. Prioritize the cli.

- Publication of the visual score, smufl and musicxml crates, so users may embed
  them in third party applications. The lib embeds `glyphnames.json` with an
  `include_str!` reaching outside the crate into `assets/smufl/metadata`, which
  a packaged crate cannot do; the file has to move into the crate first.

- Full tablature support.

- Zero panic render pipeline. Should be able to handle every imaginable MusicXML
  document, guaranteed not to panic once it passes validation. Problems in the
  document should be skipped and displayed if `--debug` is provided (such as a
  red border around a measure that has content the cli was not able to parse).

- Note horizontal position currently comes from `default-x`. The MusicXML spec
  makes that attribute optional; every test document so far has carried one,
  which is the only reason this holds. Placing a note without it means computing
  x from duration and content — real spacing — so this comes before the generic
  layout engine below rather than as part of it.

- Support for different layout engines. I imagine, for example, a layout engine
  where we can override page size and measures and systems restructure without
  reading the MusicXML print attributes. Support for single page scroll in scope.
  The current layout engine must keep support.

- Read mxl (zipped MusicXML documents).

- A separate `export` cli command: one command with several targets — mxl, older
  MusicXML document versions, and whatever else a converter is asked for.

- Support for finale, sibelius and musescore interop.

- Auto fix document issues.

- Improved Web/WASM support. The one concrete piece already scoped is
  `render_pdf` on the wasm `Score`; the blocker is that there is no system font
  database in the browser, so the font programs the PDF embeds have to be bundled
  or threaded through the constructor. See the design note at the end of
  `wasm/src/lib.rs`.

- Desktop app support — see `/platforms` skeleton.

- Mobile app support — see `/platforms` skeleton.
