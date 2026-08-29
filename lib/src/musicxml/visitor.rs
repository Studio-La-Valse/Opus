use crate::geometry::chain::Chain;
use roxmltree::Node;

/// Generates the [`Visitor`] trait and the [`Chain`] fan-out impl from one list
/// of events, so adding an event is a one-line change instead of touching the
/// trait, every `Chain` forwarder and the walker's call sites separately.
///
/// `node` events are handed the `roxmltree::Node` they fired on; `ctx` events
/// (the `exit_*` boundaries) get only the context. Every method has an empty
/// default body, so a concrete visitor implements just the events it cares about.
macro_rules! define_visitor {
    (
        node: { $($node_ev:ident),* $(,)? },
        ctx: { $($ctx_ev:ident),* $(,)? } $(,)?
    ) => {
        pub trait Visitor<C>: Sized {
            $( fn $node_ev(&mut self, _node: &Node, _ctx: &mut C) {} )*
            $( fn $ctx_ev(&mut self, _ctx: &mut C) {} )*

            fn uses<B: Visitor<C>>(self, callback: B) -> Chain<Self, B> {
                Chain::new(self, callback)
            }
        }

        impl<C, A: Visitor<C>, B: Visitor<C>> Visitor<C> for Chain<A, B> {
            $(
                fn $node_ev(&mut self, node: &Node, ctx: &mut C) {
                    self.a.$node_ev(node, ctx);
                    self.b.$node_ev(node, ctx);
                }
            )*
            $(
                fn $ctx_ev(&mut self, ctx: &mut C) {
                    self.a.$ctx_ev(ctx);
                    self.b.$ctx_ev(ctx);
                }
            )*
        }
    };
}

define_visitor! {
    node: {
        enter,
        enter_work,
        enter_defaults,
        enter_part_list,
        enter_part,
        enter_measure,
        enter_print,
        enter_attributes,
        enter_clef,
        enter_staff_details,
        enter_key,
        enter_backup,
        enter_forward,
        enter_note,
    },
    ctx: {
        exit_defaults,
        exit_note,
        exit_measure,
        exit_part,
        exit,
    },
}

pub struct DefaultVisitor {}

impl<C> Visitor<C> for DefaultVisitor {}
