use actix::{Addr};
use std::collections::HashMap;
use crate::user::User;
use crate::room::Room;
use crate::message::UnoSubscription;
use actix_web::{HttpRequest};
use actix_session::{Session};
use crate::card::{Card, all};

#[derive(Debug, Clone)]
pub struct Uno {
    pub users: HashMap<u32, User>,
    pub rooms: HashMap<u32, Room>,
    pub user_index: u32,
    pub room_index: u32,
    pub subscribers: HashMap<u32, Vec<Addr<UnoSubscription>>>,
    pub cards: Vec<Card>
}

impl Uno {
    pub fn new() -> Uno {
        Uno {
            users:HashMap::new(),
            rooms: HashMap::new(),
            user_index: 0,
            room_index: 0,
            subscribers: HashMap::new(),
            cards: all()
        }
    }
}

pub fn room_and_player(req: HttpRequest, session: Session) -> (u32, u32) {    
    let room_id = req.match_info().get("room_id").unwrap().parse::<u32>().unwrap();
    let player_id = session.get::<u32>("player_id").unwrap().unwrap_or(0);
    (room_id, player_id)
}
