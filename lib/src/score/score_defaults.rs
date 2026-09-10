use crate::musicxml::utils::{NodeUtils, ReqParse};
use crate::score::core::group_symbol::GroupSymbol;
use crate::score::part_list::tree::PartListNode;
use roxmltree::Node;
use std::collections::BTreeMap;

/// A `<score-part>`'s section/part-group assignment, returned by
/// `ScoreDefaults::lookup` for a given part id.
#[derive(Default, Clone, Copy)]
pub struct ScorePart {
    pub section: u32,
    pub part_group: u32,

    /// The `<group-symbol>` declared by the `<part-group>` this part's section
    /// came from, and by the one its part-group came from. `None` where the
    /// document named none, or where there is no such enclosing node at all --
    /// a part written at the top level of a `<part-list>` still gets a section
    /// index of its own, with nothing behind it to declare anything.
    pub section_symbol: Option<GroupSymbol>,
    pub part_group_symbol: Option<GroupSymbol>,
}

#[derive(Copy, Clone)]
pub struct Defaults {
    pub scaling_millimeters: f32,
    pub scaling_tenths: f32,
    pub page_height: f32,
    pub page_width: f32,
}

impl Default for Defaults {
    fn default() -> Defaults {
        Defaults {
            scaling_millimeters: 6.35,
            scaling_tenths: 40.,
            page_height: 1760.,
            page_width: 1360.,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PageMargins {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Default for PageMargins {
    fn default() -> PageMargins {
        PageMargins {
            left: 80.,
            right: 80.,
            top: 80.,
            bottom: 80.,
        }
    }
}

impl PageMargins {
    /// Reads one `<page-margins>`. All four edges are required by the format.
    pub fn from_mxml(node: &Node) -> PageMargins {
        PageMargins {
            left: node.req_child("left-margin").req_parse(),
            right: node.req_child("right-margin").req_parse(),
            top: node.req_child("top-margin").req_parse(),
            bottom: node.req_child("bottom-margin").req_parse(),
        }
    }
}

/// Everything a `<page-layout>` can say, all of it optional.
///
/// Optional twice over: the element's own content model makes it so --
/// `<page-height>` and `<page-width>` come as a pair or not at all, and any of
/// the three kinds of `<page-margins>` may be absent -- and inside a `<print>`
/// the omissions carry meaning. A mid-document `<page-layout>` changes what it
/// names and leaves the rest of the page geometry as it was.
#[derive(Debug, Default, Clone, Copy)]
pub struct PageLayout {
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub margins_both: Option<PageMargins>,
    pub margins_odd: Option<PageMargins>,
    pub margins_even: Option<PageMargins>,
}

impl PageLayout {
    /// Reads a `<page-layout>`, from `<defaults>` or from a `<print>`. The two
    /// mean the same thing; they differ only in what they apply to.
    pub fn from_mxml(node: &Node) -> PageLayout {
        let mut layout = PageLayout {
            width: node.get_child("page-width").map(|n| n.req_parse()),
            height: node.get_child("page-height").map(|n| n.req_parse()),
            ..Default::default()
        };

        for margins in node.children().filter(|n| n.has_tag("page-margins")) {
            let parsed = PageMargins::from_mxml(&margins);

            // `type` is optional and defaults to "both".
            match margins.get_attribute("type").unwrap_or("both") {
                "both" => layout.margins_both = Some(parsed),
                "odd" => layout.margins_odd = Some(parsed),
                "even" => layout.margins_even = Some(parsed),
                other => panic!("Invalid page-margins type '{other}'"),
            }
        }

        layout
    }

    /// The margins this layout specifies for a page of `page_number`'s parity,
    /// or `None` if it names none that apply.
    ///
    /// Resolving parity *before* folding one layout over another is what makes
    /// a later declaration win: a `<print>` carrying an untyped (`both`)
    /// `<page-margins>` has to replace the `odd` margins the defaults set, not
    /// lose to them for being in a less specific slot.
    pub fn margins_for(&self, page_number: u32) -> Option<PageMargins> {
        if page_number.is_multiple_of(2) {
            self.margins_even.or(self.margins_both)
        } else {
            self.margins_odd.or(self.margins_both)
        }
    }

    /// Folds `other` over `self`: what `other` names wins, what it leaves out
    /// keeps the value it already had.
    pub fn apply(&mut self, other: &PageLayout) {
        self.width = other.width.or(self.width);
        self.height = other.height.or(self.height);
        self.margins_both = other.margins_both.or(self.margins_both);
        self.margins_odd = other.margins_odd.or(self.margins_odd);
        self.margins_even = other.margins_even.or(self.margins_even);
    }
}

/// One page's finished geometry, resolved by [`ScoreDefaults::resolve_page`].
#[derive(Debug, Clone, Copy)]
pub struct ResolvedPage {
    pub width: f32,
    pub height: f32,
    pub margins: PageMargins,
}

#[derive(Default, Clone)]
pub struct Appearance {
    pub light_barline: Option<f32>,
    pub heavy_barline: Option<f32>,
    pub beam_thickness: Option<f32>,
    pub staff: Option<f32>,
    pub stem_thickness: Option<f32>,
    pub note_size_grace: Option<f32>,
    pub note_size_cue: Option<f32>,
}

#[derive(Clone)]
pub struct ScoreDefaults {
    pub work_title: String,

    pub defaults: Defaults,
    pub appearance: Appearance,

    /// `<defaults><page-layout>`: the page geometry the whole score starts from.
    pub page_layout: PageLayout,
    /// `<print><page-layout>`, keyed by the page it first applies to. Each entry
    /// holds only what that `<print>` named; [`resolve_page`](Self::resolve_page)
    /// folds them forward.
    pub page_overrides: BTreeMap<u32, PageLayout>,

    pub system_margin_left: f32,
    pub system_margin_right: f32,
    pub system_distance: f32,
    pub top_system_distance: f32,

    pub staff_distance: f32,

    pub part_list: Vec<PartListNode>,
}

impl ScoreDefaults {
    /// The geometry of page `page_number`.
    ///
    /// A `<page-layout>` inside a `<print>` applies from its own page onward
    /// until another one changes it, and changes only what it names. So this
    /// starts at the document defaults and folds every override up to and
    /// including this page over it in order, rather than picking a single
    /// winner. Odd- and even-page margins are folded separately and chosen from
    /// at the end, because a `<print>` that sets only the odd margins must leave
    /// the even ones alone.
    pub fn resolve_page(&self, page_number: u32) -> ResolvedPage {
        let mut width = self.page_layout.width;
        let mut height = self.page_layout.height;
        let mut margins = self.page_layout.margins_for(page_number);

        for layout in self.page_overrides.range(..=page_number).map(|(_, l)| l) {
            width = layout.width.or(width);
            height = layout.height.or(height);
            margins = layout.margins_for(page_number).or(margins);
        }

        ResolvedPage {
            width: width.unwrap_or(self.defaults.page_width),
            height: height.unwrap_or(self.defaults.page_height),
            margins: margins.unwrap_or_default(),
        }
    }

    /// Finds a part's section/part-group assignment by id, searching the
    /// part-list tree directly -- a part-list is small enough that a linear
    /// walk costs nothing, so there's no need to also keep a flattened map.
    ///
    /// The symbols each enclosing level declared are collected on the way down,
    /// which is why the descent carries them rather than the match reading them:
    /// a part knows its section's index but nothing about the node that index
    /// came from. A part written at the top level of a `<part-list>` still gets
    /// a section index of its own, with no `<part-group>` behind it to have
    /// declared anything, and comes back with both symbols absent.
    pub fn lookup(&self, part_id: &str) -> Option<ScorePart> {
        fn find(nodes: &[PartListNode], part_id: &str, enclosing: ScorePart) -> Option<ScorePart> {
            for node in nodes {
                match node {
                    PartListNode::Part {
                        id,
                        section,
                        part_group,
                        ..
                    } if id == part_id => {
                        return Some(ScorePart {
                            section: *section,
                            part_group: *part_group,
                            ..enclosing
                        });
                    }
                    PartListNode::Section {
                        symbol, children, ..
                    } => {
                        let enclosing = ScorePart {
                            section_symbol: *symbol,
                            ..enclosing
                        };
                        if let Some(found) = find(children, part_id, enclosing) {
                            return Some(found);
                        }
                    }
                    PartListNode::Group {
                        symbol, children, ..
                    } => {
                        let enclosing = ScorePart {
                            part_group_symbol: *symbol,
                            ..enclosing
                        };
                        if let Some(found) = find(children, part_id, enclosing) {
                            return Some(found);
                        }
                    }
                    PartListNode::Part { .. } => {}
                }
            }

            None
        }

        find(&self.part_list, part_id, ScorePart::default())
    }

    /// Ensures a part is present in the part-list, registering a bare
    /// top-level entry (no name/abbr, default section/part-group 0) if the
    /// id was never declared in `<part-list>`.
    pub fn ensure_part(&mut self, part_id: &str) {
        if self.lookup(part_id).is_some() {
            return;
        }

        self.part_list.push(PartListNode::Part {
            id: part_id.to_string(),
            name: String::new(),
            abbr: String::new(),
            section: 0,
            part_group: 0,
        });
    }
}

impl Default for ScoreDefaults {
    fn default() -> ScoreDefaults {
        ScoreDefaults {
            work_title: Default::default(),
            defaults: Default::default(),
            appearance: Default::default(),

            page_layout: Default::default(),
            page_overrides: BTreeMap::new(),

            system_margin_left: 0.,
            system_margin_right: 0.,
            system_distance: 130.,
            top_system_distance: 70.,

            staff_distance: 80.,

            part_list: Vec::new(),
        }
    }
}
