/// Instances of types that implement this trait can be reset to the state they
/// were in when they were constructed
pub trait Reset {
    /// Reset `self` to the state it was in when it was constructed
    fn reset(&mut self);
}
