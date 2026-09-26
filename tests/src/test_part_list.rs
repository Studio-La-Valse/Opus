#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use lib::musicxml::validate::ValidationCtx;
    use lib::musicxml::validation_issue::{Severity, ValidationIssue};
    use lib::musicxml::visitor::{DefaultVisitor, Visitor};
    use lib::musicxml::visitors::validators::part_consistency_visitor::PartConsistencyVisitor;
    use lib::musicxml::visitors::validators::position_visitor::PositionVisitor;
    use lib::musicxml::walker::Walker;
    use lib::score::core::group_symbol::GroupSymbol;
    use lib::score::engrave::walk_document;

    use lib::score::part_list::builder::build_part_list;
    use lib::score::part_list::display::format_part_list_tree;
    use lib::score::part_list::tree::PartListNode;

    const ACTOR_PRELUDE: &str = "assets/xmlsamples/ActorPreludeSample.musicxml";

    fn fixture(relative: &str) -> String {
        read_to_string(format!("{}/../{relative}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative}: {e}"))
    }

    fn parse(xml: &str) -> roxmltree::Document<'_> {
        roxmltree::Document::parse_with_options(
            xml,
            roxmltree::ParsingOptions {
                allow_dtd: true,
                ..Default::default()
            },
        )
        .expect("failed to parse test document")
    }

    fn build_str(xml: &str) -> Vec<PartListNode> {
        let doc = roxmltree::Document::parse(xml).expect("failed to parse test part-list");
        build_part_list(&doc.root_element())
    }

    fn build_actor_prelude() -> Vec<PartListNode> {
        let xml = fixture(ACTOR_PRELUDE);
        let doc = parse(&xml);
        let part_list = doc
            .descendants()
            .find(|n| n.has_tag_name("part-list"))
            .expect("ActorPreludeSample has no <part-list>");

        build_part_list(&part_list)
    }

    /// (name, symbol, children) of a `Section`, panicking if the node is anything else.
    fn section(node: &PartListNode) -> (Option<&str>, Option<GroupSymbol>, &[PartListNode]) {
        match node {
            PartListNode::Section {
                name,
                symbol,
                children,
                ..
            } => (name.as_deref(), *symbol, children.as_slice()),
            _ => panic!("expected a section"),
        }
    }

    /// (name, symbol, children) of a `Group`, panicking if the node is anything else.
    fn group(node: &PartListNode) -> (Option<&str>, Option<GroupSymbol>, &[PartListNode]) {
        match node {
            PartListNode::Group {
                name,
                symbol,
                children,
                ..
            } => (name.as_deref(), *symbol, children.as_slice()),
            _ => panic!("expected a part-group"),
        }
    }

    fn part_ids(nodes: &[PartListNode]) -> Vec<&str> {
        nodes
            .iter()
            .filter_map(|n| match n {
                PartListNode::Part { id, .. } => Some(id.as_str()),
                _ => None,
            })
            .collect()
    }

    /// Two starts in a row carry no positional evidence of which one encloses
    /// the other, so the bracket has to end up outside the brace even when the
    /// exporter wrote them the other way round.
    #[test]
    fn braced_start_before_bracket_start_is_reordered() {
        let nodes = build_str(
            r#"<part-list>
                <part-group type="start">
                    <group-name>Horns in F</group-name>
                    <group-symbol>brace</group-symbol>
                </part-group>
                <part-group type="start">
                    <group-symbol>bracket</group-symbol>
                </part-group>
                <score-part id="P1"><part-name>1 2</part-name></score-part>
                <part-group type="stop"/>
                <score-part id="P2"><part-name>Tuba</part-name></score-part>
                <part-group type="stop"/>
            </part-list>"#,
        );

        assert_eq!(nodes.len(), 1);
        let (name, symbol, children) = section(&nodes[0]);
        assert_eq!(name, None, "a bracket carries no name here");
        assert_eq!(symbol, Some(GroupSymbol::Bracket));

        let (name, symbol, _) = group(&children[0]);
        assert_eq!(name, Some("Horns in F"));
        assert_eq!(symbol, Some(GroupSymbol::Brace));
    }

    /// The same run written the right way round must come out unchanged.
    #[test]
    fn bracket_start_before_braced_start_is_left_alone() {
        let nodes = build_str(
            r#"<part-list>
                <part-group type="start">
                    <group-symbol>bracket</group-symbol>
                </part-group>
                <part-group type="start">
                    <group-name>Horns in F</group-name>
                    <group-symbol>brace</group-symbol>
                </part-group>
                <score-part id="P1"><part-name>1 2</part-name></score-part>
                <part-group type="stop"/>
                <part-group type="stop"/>
            </part-list>"#,
        );

        let (name, symbol, children) = section(&nodes[0]);
        assert_eq!(name, None);
        assert_eq!(symbol, Some(GroupSymbol::Bracket));

        let (name, symbol, _) = group(&children[0]);
        assert_eq!(name, Some("Horns in F"));
        assert_eq!(symbol, Some(GroupSymbol::Brace));
    }

    /// A `<score-part>` between two starts *is* positional evidence, so the
    /// order those two were written in stands, symbols notwithstanding.
    #[test]
    fn starts_split_by_a_part_keep_document_order() {
        let nodes = build_str(
            r#"<part-list>
                <part-group type="start">
                    <group-name>Outer</group-name>
                    <group-symbol>brace</group-symbol>
                </part-group>
                <score-part id="P1"><part-name>Timpani</part-name></score-part>
                <part-group type="start">
                    <group-symbol>bracket</group-symbol>
                </part-group>
                <score-part id="P2"><part-name>Cymbal</part-name></score-part>
                <part-group type="stop"/>
                <part-group type="stop"/>
            </part-list>"#,
        );

        let (name, symbol, children) = section(&nodes[0]);
        assert_eq!(name, Some("Outer"));
        assert_eq!(symbol, Some(GroupSymbol::Brace));

        let (_, symbol, _) = group(&children[1]);
        assert_eq!(symbol, Some(GroupSymbol::Bracket));
    }

    /// A symbol-less group is the innermost of a run: nothing encloses less
    /// than nothing.
    #[test]
    fn symbol_less_start_sinks_below_a_bracket() {
        let nodes = build_str(
            r#"<part-list>
                <part-group type="start">
                    <group-name>1 2</group-name>
                </part-group>
                <part-group type="start">
                    <group-symbol>bracket</group-symbol>
                </part-group>
                <score-part id="P1"><part-name>Flutes</part-name></score-part>
                <part-group type="stop"/>
                <part-group type="stop"/>
            </part-list>"#,
        );

        let (name, symbol, children) = section(&nodes[0]);
        assert_eq!(name, None);
        assert_eq!(symbol, Some(GroupSymbol::Bracket));

        let (name, symbol, _) = group(&children[0]);
        assert_eq!(name, Some("1 2"));
        assert_eq!(symbol, None);
    }

    /// An explicit `<group-symbol>none</group-symbol>` is not the same thing as
    /// an absent one: the first says "draw nothing", the second says nothing at
    /// all and lets a default apply. Both rank innermost, so they nest the same
    /// way, but only one of them survives into the tree as a value.
    #[test]
    fn an_explicit_none_is_kept_apart_from_an_absent_symbol() {
        let nodes = build_str(
            r#"<part-list>
                <part-group type="start">
                    <group-symbol>bracket</group-symbol>
                </part-group>
                <part-group type="start">
                    <group-symbol>none</group-symbol>
                </part-group>
                <score-part id="P1"><part-name>Flutes</part-name></score-part>
                <part-group type="stop"/>
                <part-group type="stop"/>
            </part-list>"#,
        );

        let (_, symbol, children) = section(&nodes[0]);
        assert_eq!(symbol, Some(GroupSymbol::Bracket));

        let (_, symbol, _) = group(&children[0]);
        assert_eq!(
            symbol,
            Some(GroupSymbol::None),
            "an explicit 'none' is a declared value, not an absent one"
        );
    }

    /// The two symbols the engine could not draw before must still read back off
    /// the document, since honouring them is the whole point of the type.
    #[test]
    fn line_and_square_are_read_like_any_other_symbol() {
        let nodes = build_str(
            r#"<part-list>
                <part-group type="start">
                    <group-symbol>line</group-symbol>
                </part-group>
                <score-part id="P1"><part-name>Violin I</part-name></score-part>
                <score-part id="P2"><part-name>Violin II</part-name></score-part>
                <part-group type="stop"/>
                <part-group type="start">
                    <group-symbol>square</group-symbol>
                </part-group>
                <score-part id="P3"><part-name>Viola</part-name></score-part>
                <part-group type="stop"/>
            </part-list>"#,
        );

        let (_, symbol, _) = section(&nodes[0]);
        assert_eq!(symbol, Some(GroupSymbol::Line));

        let (_, symbol, _) = section(&nodes[1]);
        assert_eq!(symbol, Some(GroupSymbol::Square));
    }

    /// `line` and `square` are bracket-weight symbols, so a run that pairs one
    /// with a brace has to nest the same way `bracket` would.
    #[test]
    fn a_line_start_encloses_a_braced_start() {
        let nodes = build_str(
            r#"<part-list>
                <part-group type="start">
                    <group-symbol>brace</group-symbol>
                </part-group>
                <part-group type="start">
                    <group-symbol>line</group-symbol>
                </part-group>
                <score-part id="P1"><part-name>Organ</part-name></score-part>
                <part-group type="stop"/>
                <part-group type="stop"/>
            </part-list>"#,
        );

        let (_, symbol, children) = section(&nodes[0]);
        assert_eq!(symbol, Some(GroupSymbol::Line));

        let (_, symbol, _) = group(&children[0]);
        assert_eq!(symbol, Some(GroupSymbol::Brace));
    }

    /// Parsing trims and lowercases, so surrounding whitespace -- which XML
    /// pretty-printers add freely -- names the symbol it looks like it names.
    /// Before the value was a type, a padded `<group-symbol>` matched no arm of
    /// the nesting comparison and silently sank to the bottom of its run.
    #[test]
    fn a_padded_symbol_names_the_symbol_it_looks_like() {
        let nodes = build_str(
            r#"<part-list>
                <part-group type="start">
                    <group-symbol>
                        Bracket
                    </group-symbol>
                </part-group>
                <score-part id="P1"><part-name>Flute</part-name></score-part>
                <part-group type="stop"/>
            </part-list>"#,
        );

        let (_, symbol, _) = section(&nodes[0]);
        assert_eq!(symbol, Some(GroupSymbol::Bracket));
    }

    /// The tree is rendered back with the document's own spelling, which is what
    /// keeps the build log readable against the file it describes.
    #[test]
    fn every_symbol_renders_as_its_musicxml_spelling() {
        let spellings = [
            (GroupSymbol::None, "none"),
            (GroupSymbol::Brace, "brace"),
            (GroupSymbol::Bracket, "bracket"),
            (GroupSymbol::Line, "line"),
            (GroupSymbol::Square, "square"),
        ];

        for (symbol, spelled) in spellings {
            assert_eq!(symbol.to_string(), spelled);
            assert_eq!(
                spelled.parse::<GroupSymbol>().unwrap(),
                symbol,
                "'{spelled}' must parse back to what it prints"
            );
        }
    }

    /// ActorPreludeSample's brass block: the export names the outer group
    /// "Horns in F" and opens the bracket second, and its `number` attributes
    /// disagree with its own start/stop pairing. The section must still come
    /// out as the unnamed bracket spanning P8..P13, with the horn pair braced
    /// and named inside it.
    #[test]
    fn actor_prelude_brass_section_is_the_bracket_not_the_horns() {
        let nodes = build_actor_prelude();

        let (name, symbol, children) = section(&nodes[1]);
        assert_eq!(name, None, "sections in this score are unnamed");
        assert_eq!(symbol, Some(GroupSymbol::Bracket));

        let (name, symbol, horns) = group(&children[0]);
        assert_eq!(name, Some("Horns in F"));
        assert_eq!(symbol, Some(GroupSymbol::Brace));
        assert_eq!(part_ids(horns), vec!["P8", "P9"]);

        // The tuba sits directly in the section, after the three inner groups.
        assert_eq!(part_ids(children), vec!["P13"]);
        assert_eq!(children.len(), 4);
    }

    /// The rest of the same part-list must be untouched by that reordering:
    /// every section is an unnamed bracket, and the names stay on the groups.
    #[test]
    fn actor_prelude_sections_are_all_unnamed_brackets() {
        let nodes = build_actor_prelude();

        let sections: Vec<_> = nodes
            .iter()
            .filter(|n| matches!(n, PartListNode::Section { .. }))
            .map(section)
            .collect();

        assert_eq!(sections.len(), 4);
        for (name, symbol, _) in &sections {
            assert_eq!(*name, None);
            assert_eq!(*symbol, Some(GroupSymbol::Bracket));
        }

        let (_, _, percussion) = sections[2];
        assert_eq!(part_ids(percussion), vec!["P14"]);
        let (name, symbol, _) = group(&percussion[1]);
        assert_eq!(name, Some("Percussion"));
        assert_eq!(symbol, Some(GroupSymbol::Brace));
    }

    // ------------------------------------------------------------- validation

    /// Runs the validation walk that owns the `<part-group>` balance rules.
    fn validate(xml: &str) -> Vec<ValidationIssue> {
        let doc = parse(xml);
        let mut ctx = ValidationCtx::default();
        let visitor = DefaultVisitor {}.uses(PartConsistencyVisitor::default());

        Walker::new(visitor).walk(&doc, &mut ctx);

        ctx.issues
    }

    /// The messages of every issue at `severity`, so a test can assert on the
    /// rule it cares about without listing the walk's Info chatter.
    fn messages(issues: &[ValidationIssue], severity: Severity) -> Vec<&str> {
        issues
            .iter()
            .filter(|i| i.severity == severity)
            .map(|i| i.message.as_str())
            .collect()
    }

    /// A minimal score whose `<part-list>` body is `part_list`, with one part so
    /// the unrelated "score has no <part> elements" rule stays quiet.
    fn score_with_part_list(part_list: &str) -> String {
        format!(
            r#"<score-partwise>
                <part-list>{part_list}</part-list>
                <part id="P1"><measure number="1"/></part>
            </score-partwise>"#
        )
    }

    #[test]
    fn a_stop_without_a_start_is_an_error() {
        let issues = validate(&score_with_part_list(
            r#"<score-part id="P1"><part-name>Piano</part-name></score-part>
               <part-group type="stop"/>"#,
        ));

        assert_eq!(
            messages(&issues, Severity::Error),
            vec!["<part-group type=\"stop\"> has no matching start"]
        );
    }

    #[test]
    fn an_unclosed_group_is_a_warning() {
        let issues = validate(&score_with_part_list(
            r#"<part-group type="start"><group-symbol>bracket</group-symbol></part-group>
               <part-group type="start"><group-symbol>brace</group-symbol></part-group>
               <score-part id="P1"><part-name>Piano</part-name></score-part>"#,
        ));

        assert_eq!(
            messages(&issues, Severity::Warning),
            vec!["<part-list> leaves 2 <part-group>(s) unclosed"]
        );
        assert!(messages(&issues, Severity::Error).is_empty());
    }

    /// `build_part_list` reads `type` with `req_attribute` and dies without it,
    /// which is only acceptable because validation names the cause first.
    #[test]
    fn a_part_group_without_a_type_is_an_error() {
        let issues = validate(&score_with_part_list(
            r#"<part-group><group-symbol>bracket</group-symbol></part-group>
               <score-part id="P1"><part-name>Piano</part-name></score-part>"#,
        ));

        assert_eq!(
            messages(&issues, Severity::Error),
            vec!["<part-group> is missing required 'type' attribute"]
        );
    }

    #[test]
    #[should_panic(expected = "type")]
    fn the_builder_refuses_a_part_group_without_a_type() {
        build_str(
            r#"<part-list><part-group><group-symbol>bracket</group-symbol></part-group></part-list>"#,
        );
    }

    /// The other half of the same bargain, for `<score-part>`'s `id`.
    #[test]
    fn a_score_part_without_an_id_is_an_error() {
        let issues = validate(&score_with_part_list(
            r#"<score-part><part-name>Piano</part-name></score-part>"#,
        ));

        assert!(
            messages(&issues, Severity::Error)
                .contains(&"<score-part> is missing required 'id' attribute")
        );
    }

    #[test]
    #[should_panic(expected = "id")]
    fn the_builder_refuses_a_score_part_without_an_id() {
        build_str(
            r#"<part-list><score-part><part-name>Piano</part-name></score-part></part-list>"#,
        );
    }

    /// The whole point of reading start/stop order instead of `number`:
    /// ActorPreludeSample's part-groups are perfectly balanced even though its
    /// numbering is not, so validation must find nothing to complain about.
    #[test]
    fn actor_prelude_reports_no_part_group_issues() {
        let issues = validate(&fixture(ACTOR_PRELUDE));

        assert!(
            !issues
                .iter()
                .any(|i| i.severity != Severity::Info && i.message.contains("part-group")),
            "unexpected part-group issues: {:?}",
            issues
                .iter()
                .filter(|i| i.severity != Severity::Info)
                .collect::<Vec<_>>()
        );
    }

    /// A validation walk is a verdict on a document, so every issue it produces
    /// is a defect. Narration lives on the build walk instead.
    #[test]
    fn the_validation_walk_reports_only_defects() {
        let musicxml = fixture(ACTOR_PRELUDE);
        let doc = parse(&musicxml);
        let mut ctx = ValidationCtx::default();
        let visitor = DefaultVisitor {}
            .uses(PartConsistencyVisitor::default())
            .uses(PositionVisitor::default());

        Walker::new(visitor).walk(&doc, &mut ctx);

        assert!(ctx.issues.iter().all(|i| i.severity != Severity::Info));
    }

    // ------------------------------------------------------------ build logging

    /// Walks a document for real and returns what the build walk logged.
    fn log_build(xml: &str) -> Vec<ValidationIssue> {
        let document = parse(xml);

        let (_score, _defaults, messages) = walk_document(&document, &mut |_| {});

        assert!(
            messages.iter().all(|m| m.severity == Severity::Info),
            "the build logger reports nothing but Info"
        );

        messages
    }

    fn log_actor_prelude() -> Vec<ValidationIssue> {
        log_build(&fixture(ACTOR_PRELUDE))
    }

    #[test]
    fn the_build_logger_reports_the_tree_it_built() {
        let messages = log_actor_prelude();
        let tree = format_part_list_tree(&build_actor_prelude());

        assert_eq!(
            messages.iter().filter(|m| m.message == tree).count(),
            1,
            "the tree is logged exactly once, as built"
        );
        assert!(tree.contains("part-group 0 \"Horns in F\" (brace)"));
    }

    /// Everything that used to be said on the validation walk is said here now,
    /// in the same order, with the tree between reading the part-list and
    /// walking the parts.
    #[test]
    fn the_build_logger_narrates_the_whole_walk() {
        let messages = log_actor_prelude();
        let logged: Vec<&str> = messages.iter().map(|m| m.message.as_str()).collect();
        let tree = format_part_list_tree(&build_actor_prelude());

        assert_eq!(
            logged,
            vec![
                "Now entering score-partwise",
                "Collecting parts...",
                tree.as_str(),
                "Now traversing parts...",
                "Done, now gracefully exiting score-partwise",
                "Found 22 part(s) and 902 measure(s) total",
                "found 2945 note(s) (757 rest(s)), 55 backup(s), 8 forward(s)",
            ]
        );
    }

    /// It narrates whatever it is given, rather than assuming score-partwise.
    #[test]
    fn the_build_logger_names_the_root_it_actually_found() {
        let messages = log_build("<score-timewise><part-list/></score-timewise>");
        let logged: Vec<&str> = messages.iter().map(|m| m.message.as_str()).collect();

        assert_eq!(logged.first(), Some(&"Now entering score-timewise"));
        assert!(logged.contains(&"Done, now gracefully exiting score-timewise"));
    }

    /// The tree is positioned at `<part-list>`, so the CLI resolves it to that
    /// element's line and column rather than to the top of the file.
    #[test]
    fn the_build_logger_positions_the_tree_at_the_part_list() {
        let musicxml = fixture(ACTOR_PRELUDE);
        let document = parse(&musicxml);
        let part_list = document
            .descendants()
            .find(|n| n.has_tag_name("part-list"))
            .expect("ActorPreludeSample has no <part-list>");

        let messages = log_actor_prelude();
        let tree = messages
            .iter()
            .find(|m| m.message.contains("├──"))
            .expect("the tree was never logged");

        assert_eq!(tree.at, part_list.range().start);
    }
}
