#[derive(Clone, PartialEq, Debug)]
pub struct Salt;

impl Default for Salt {
    fn default() -> Self {
        Self::new()
    }
}

impl Salt {
    pub fn new() -> Self {
        Self {}
    }
}
