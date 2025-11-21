#[derive(Clone, PartialEq, Debug)]
pub struct Sand;

impl Default for Sand {
    fn default() -> Self {
        Self::new()
    }
}

impl Sand {
    pub fn new() -> Self {
        Self {}
    }
}
