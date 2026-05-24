pub struct Chain<A, B> {
    pub a: A,
    pub b: B,
}

impl<A, B> Chain<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Chain { a, b }
    }
}
