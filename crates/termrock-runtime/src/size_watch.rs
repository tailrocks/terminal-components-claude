use crate::event::Input;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SizeWatch {
    known: Option<(u16, u16)>,
}

impl SizeWatch {
    pub fn new(size: Option<(u16, u16)>) -> Self {
        Self { known: size }
    }

    pub fn observe(&mut self, input: &Input) {
        if let Input::Resize(width, height) = input {
            self.known = Some((*width, *height));
        }
    }

    pub fn check(&mut self, actual: (u16, u16)) -> Option<Input> {
        if self.known == Some(actual) {
            return None;
        }
        self.known = Some(actual);
        Some(Input::Resize(actual.0, actual.1))
    }
}
