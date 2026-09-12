Hello please consider tie_arranger.rs. 

As of right now, ties are a member of systems, arranged by score. This seems necessary, as notes may be tied between systems (and pages). However in my mind, ties are bound strictly to parts - in fact even to notes and their following note. 

Therefore I would like to propose the following refactor: we get rid of the ties member on System. We instead give every Note two new fields: tie: Option<Tie> and courtesy_tie: Option<Tie>. Which notes are tied together moves to ContentVisitor. When two adjecent notes in the same system are tied, the left note gains Some(tie) on the member. When tracked across systems: the left gains a Some(tie) value, and the right a Some(courtesy_tie). 

Tie should implement an arrange() function, accepting a some metrics, like width, height etc. The function arrange_ties in tie_arranger should arrange on Part, not on Score. A part should have enough information to determine the available space for the length of a courtesy tie, or a tie that has not target on the right (so the target is the end of the part). For each note searching for the next should be simplified as well: Notes may not be tied cross chord which means the target can only be the next - greatly simplifying the search radius. 

I think if we accomplish this, we can get rid of the following: `notes: Vec<(NoteId, Note)>` on Chord becomes `Vec<note>` (NoteId is not required anymore because tied notes are tracked by ContentVisitor), and `ties: Vec<Tie>` on System (moved to options on Note struct). 

Go ahead and implement, consult me if LOC explodes. Feel free to push back if you encounter breaking changes or changes that may block future functionality.

If all goes to plan I think we will have a net negative LOC pr with byte by byte equal render results. 

One caveat is that if we track cross system ties through the ContentVisitor, we cannot implement a new generic layout engine, which for example lays out all measures in one continuous system.
