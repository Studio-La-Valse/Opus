#[cfg(test)]
mod tests {
    use lib::score::visual::stem::BeamType;

    /// The five values MusicXML's `<beam>` may hold. The hook spellings are the
    /// point: they are two words each, and only one of them used to be handled.
    #[test]
    fn every_musicxml_beam_value_parses() {
        assert_eq!(BeamType::from("begin"), BeamType::Start);
        assert_eq!(BeamType::from("continue"), BeamType::Continue);
        assert_eq!(BeamType::from("end"), BeamType::End);
        assert_eq!(BeamType::from("forward hook"), BeamType::HookStart);
        assert_eq!(BeamType::from("backward hook"), BeamType::HookEnd);
    }

    /// A forward hook points to the right of its stem and a backward hook to the
    /// left, so the two must not be conflated -- swapping them would draw the
    /// hook on the wrong side rather than fail.
    #[test]
    fn the_two_hooks_are_distinct() {
        assert_ne!(
            BeamType::from("forward hook"),
            BeamType::from("backward hook")
        );
    }
}
