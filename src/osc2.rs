pub struct Osc2 {}

impl Osc2 {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize() {
        let _osc2 = Osc2::new();
    }
}
