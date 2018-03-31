#[derive(Clone, Copy, Debug)]
pub struct TargetType {
    byte: u8,
}

pub enum TargetTypes {
    Ground,
    Air,
    Water,
    Underwater,
    Hover,
}

impl TargetType {
    pub fn new() -> TargetType {
        TargetType { byte: 0 }
    }

    pub fn new_all_set() -> TargetType {
        TargetType { byte: 0b11111111 }
    }

    pub fn set(self, target_types: TargetTypes) -> TargetType {
        let ix = target_types as usize;
        TargetType { byte: self.byte | 1 << ix }
    }

    pub fn get(self, target_types: TargetTypes) -> bool {
        let ix = target_types as usize;
        self.byte & (1 << ix) == (1 << ix)
    }

    pub fn has_a_match(self, other: TargetType) -> bool {
        self.byte & other.byte > 0
    }
}