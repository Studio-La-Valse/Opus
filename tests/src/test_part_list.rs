#[cfg(test)]
mod tests {
    use std::fs::read_to_string;

    use lib::score::part_list::builder::{PartGroupAction, PartListBuilder};
    use lib::score::part_list::tree::PartListNode;

    const ACTOR_PRELUDE: &str = "assets/xmlsamples/ActorPreludeSample.musicxml";

    fn fixture(relative: &str) -> String {
        read_to_string(format!("{}/../{relative}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative}: {e}"))
    }

    /// Runs the builder over a `<part-list>` element, the graceful way the
    /// validator does (no panics on missing attributes).
    fn build(part_list: &roxmltree::Node) -> Vec<PartListNode> {
        let mut builder = PartListBuilder::default();

        builder.build(
            part_list,
            |n| match n.attribute("type") {
                Some("start") => PartGroupAction::Start,
                Some("stop") => PartGroupAction::Stop,
                _ => PartGroupAction::Other,
            },
            |n| n.attribute("id").map(|id| id.to_string()),
        );

        builder.finish()
    }

    fn build_str(xml: &str) -> Vec<PartListNode> {
        let doc = roxmltree::Document::parse(xml).expect("failed to parse test part-list");
        build(&doc.root_element())
    }

    fn build_actor_prelude() -> Vec<PartListNode> {
        let xml = fixture(ACTOR_PRELUDE);
        let doc = roxmltree::Document::parse_with_options(
            &xml,
            roxmltree::ParsingOptions {
                allow_dtd: true,
                ..Default::default()
            },
        )
        .expect("failed to parse ActorPreludeSample");
        let part_list = doc
            .descendants()
            .find(|n| n.has_tag_name("part-list"))
            .expect("ActorPreludeSample has no <part-list>");

        build(&part_list)
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
}
