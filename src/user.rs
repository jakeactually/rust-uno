use crate::card::Card;
use serde::Serialize;

#[derive(Clone, Serialize, Debug)]
pub struct User {
    pub name: String,
    pub id: u32,
    pub hand: Vec<(u8, Card)>,
    pub drawed: bool,
}

impl User {
    pub fn new(name: String, id: u32) -> User {
        User {
            name,
            id,
            hand: vec![],
            drawed: false,
        }
    }
}
