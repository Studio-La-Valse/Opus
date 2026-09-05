#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use lib::musicxml::validate::ValidationCtx;
    use lib::musicxml::validation_issue::{Severity, ValidationIssue};
    use lib::musicxml::visitor::{DefaultVisitor, Visitor};
    use lib::musicxml::visitors::logging_visitor::LoggingVisitor;
    use lib::musicxml::visitors::part_consistency_visitor::PartConsistencyVisitor;
    use lib::musicxml::walker::Walker;
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

    /// (name, brace, children) of a `Section`, panicking if the node is anything else.
    fn section(node: &PartListNode) -> (Option<&str>, Option<&str>, &[PartListNode]) {
        match node {
            PartListNode::Section {
                name,
                brace,
                children,
                ..
            } => (name.as_deref(), brace.as_deref(), children.as_slice()),
            _ => panic!("expected a section"),
        }
    }

    /// (name, brace, children) of a `Group`, panicking if the node is anything else.
    fn group(node: &PartListNode) -> (Option<&str>, Option<&str>, &[PartListNode]) {
        match node {
            PartListNode::Group {
                name,
                brace,
                children,
                ..
            } => (name.as_deref(), brace.as_deref(), children.as_slice()),
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
        let (name, brace, children) = section(&nodes[0]);
        assert_eq!(name, None, "a bracket carries no name here");
        assert_eq!(brace, Some("bracket"));

        let (name, brace, _) = group(&children[0]);
        assert_eq!(name, Some("Horns in F"));
        assert_eq!(brace, Some("brace"));
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

        let (name, brace, children) = section(&nodes[0]);
        assert_eq!(name, None);
        assert_eq!(brace, Some("bracket"));

        let (name, brace, _) = group(&children[0]);
        assert_eq!(name, Some("Horns in F"));
        assert_eq!(brace, Some("brace"));
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

        let (name, brace, children) = section(&nodes[0]);
        assert_eq!(name, Some("Outer"));
        assert_eq!(brace, Some("brace"));

        let (_, brace, _) = group(&children[1]);
        assert_eq!(brace, Some("bracket"));
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

        let (name, brace, children) = section(&nodes[0]);
        assert_eq!(name, None);
        assert_eq!(brace, Some("bracket"));

        let (name, brace, _) = group(&children[0]);
        assert_eq!(name, Some("1 2"));
        assert_eq!(brace, None);
    }

    /// ActorPreludeSample's brass block: the export names the outer group
    /// "Horns in F" and opens the bracket second, and its `number` attributes
    /// disagree with its own start/stop pairing. The section must still come
    /// out as the unnamed bracket spanning P8..P13, with the horn pair braced
    /// and named inside it.
    #[test]
    fn actor_prelude_brass_section_is_the_bracket_not_the_horns() {
        let nodes = build_actor_prelude();

        let (name, brace, children) = section(&nodes[1]);
        assert_eq!(name, None, "sections in this score are unnamed");
        assert_eq!(brace, Some("bracket"));

        let (name, brace, horns) = group(&children[0]);
        assert_eq!(name, Some("Horns in F"));
        assert_eq!(brace, Some("brace"));
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
        for (name, brace, _) in &sections {
            assert_eq!(*name, None);
            assert_eq!(*brace, Some("bracket"));
        }

        let (_, _, percussion) = sections[2];
        assert_eq!(part_ids(percussion), vec!["P14"]);
        let (name, brace, _) = group(&percussion[1]);
        assert_eq!(name, Some("Percussion"));
        assert_eq!(brace, Some("brace"));
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

    /// The consistency visitor reports defects and narrates nothing: no tree,
    /// no running commentary, nothing at Info at all. That is the logging
    /// visitor's job now.
    #[test]
    fn the_consistency_visitor_does_not_narrate() {
        let issues = validate(&fixture(ACTOR_PRELUDE));

        assert!(issues.iter().all(|i| i.severity != Severity::Info));
    }

    // ---------------------------------------------------------------- logging

    /// Runs the logging visitor on its own, so a test sees its narration and
    /// nothing else.
    fn log(xml: &str) -> Vec<String> {
        let doc = parse(xml);
        let mut ctx = ValidationCtx::default();
        let visitor = DefaultVisitor {}.uses(LoggingVisitor::default());

        Walker::new(visitor).walk(&doc, &mut ctx);

        assert!(
            ctx.issues.iter().all(|i| i.severity == Severity::Info),
            "the logging visitor reports nothing but Info"
        );

        ctx.issues.into_iter().map(|i| i.message).collect()
    }

    #[test]
    fn the_logging_visitor_reports_the_built_tree() {
        let logged = log(&fixture(ACTOR_PRELUDE));
        let tree = format_part_list_tree(&build_actor_prelude());

        assert_eq!(
            logged.iter().filter(|m| **m == tree).count(),
            1,
            "the tree is logged exactly once, as built"
        );
        assert!(tree.contains("part-group 0 \"Horns in F\" (brace)"));
    }

    /// The tree is reported as reading the part-list finishes, after the
    /// "Collecting parts..." that announces it and before the parts are walked.
    #[test]
    fn the_tree_is_logged_between_collecting_and_traversing() {
        let logged = log(&fixture(ACTOR_PRELUDE));
        let position = |needle: &str| {
            logged
                .iter()
                .position(|m| m.starts_with(needle))
                .unwrap_or_else(|| panic!("never logged: {needle}"))
        };

        assert!(position("Collecting parts...") < position("score-partwise\n"));
        assert!(position("score-partwise\n") < position("Now traversing parts..."));
    }

    #[test]
    fn the_logging_visitor_narrates_the_whole_walk() {
        let logged = log(&fixture(ACTOR_PRELUDE));

        assert_eq!(
            logged.first().map(String::as_str),
            Some("Now entering score-partwise")
        );
        assert_eq!(
            &logged[logged.len() - 2..],
            &[
                "Done, now gracefully exiting score-partwise".to_string(),
                "Found 22 part(s) and 902 measure(s) total".to_string(),
            ]
        );
    }

    /// It narrates whatever it is given, rather than assuming score-partwise.
    #[test]
    fn the_logging_visitor_names_the_root_it_actually_found() {
        let logged = log("<score-timewise><part-list/></score-timewise>");

        assert_eq!(
            logged.first().map(String::as_str),
            Some("Now entering score-timewise")
        );
        assert!(logged.contains(&"Done, now gracefully exiting score-timewise".to_string()));
    }

    /// A part-list the builder cannot fully read still logs the tree it could
    /// get out of it, rather than bringing the walk down.
    #[test]
    fn a_malformed_part_list_is_logged_not_fatal() {
        let logged = log(&score_with_part_list(
            r#"<part-group><group-symbol>bracket</group-symbol></part-group>
               <score-part><part-name>No id</part-name></score-part>
               <score-part id="P1"><part-name>Piano</part-name></score-part>"#,
        ));

        assert!(logged.iter().any(|m| m.contains("P1 \"Piano\"")));
        assert!(!logged.iter().any(|m| m.contains("No id")));
    }
}
