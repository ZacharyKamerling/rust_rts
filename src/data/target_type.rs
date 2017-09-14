#[derive(Clone, Copy, Debug)]
pub struct TargetType {
    byte: u8,
}

impl TargetType {
    pub fn new() -> TargetType {
        TargetType { byte: 0 }
    }

    pub fn new_all_set() -> TargetType {
        TargetType { byte: 0b11111111 }
    }

    //1
    pub fn set_ground(self) -> TargetType {
        TargetType { byte: self.byte | 1 }
    }

    pub fn ground(self) -> bool {
        self.byte & 1 == 1
    }

    //2
    pub fn set_air(self) -> TargetType {
        TargetType { byte: self.byte | (1 << 1) }
    }

    pub fn air(self) -> bool {
        self.byte & (1 << 1) == (1 << 1)
    }

    //3
    pub fn set_water(self) -> TargetType {
        TargetType { byte: self.byte | (1 << 2) }
    }

    pub fn water(self) -> bool {
        self.byte & (1 << 2) == (1 << 2)
    }

    //4
    pub fn set_structure(self) -> TargetType {
        TargetType { byte: self.byte | (1 << 3) }
    }

    pub fn structure(self) -> bool {
        self.byte & (1 << 3) == (1 << 3)
    }

    //5
    pub fn set_underwater(self) -> TargetType {
        TargetType { byte: self.byte | (1 << 4) }
    }

    pub fn underwater(self) -> bool {
        self.byte & (1 << 4) == (1 << 4)
    }

    pub fn has_a_match(self, other: TargetType) -> bool {
        self.byte & other.byte > 0
    }
}
