use near_sdk::{near_bindgen};

#[near_bindgen]
pub struct Contract {
    pub epoch: u16;
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            epoch: 0u16,
        }
    }
}