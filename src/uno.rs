use crate::card::{all, Card};
use crate::room::Room;
use crate::user::User;
use actix_session::Session;
use actix_web::HttpRequest;
use std::{
    collections::HashMap,
    fmt::{self, Debug},
};

#[derive(Debug, Clone)]
pub struct Uno {
    pub users: HashMap<u32, User>,
    pub rooms: HashMap<u32, Room>,
    pub user_index: u32,
    pub room_index: u32,
    pub subscribers: HashMap<u32, Vec<UnoSocket>>,
    pub cards: Vec<Card>,
}

impl Uno {
    pub fn new() -> Uno {
        Uno {
            users: HashMap::new(),
            rooms: HashMap::new(),
            user_index: 0,
            room_index: 0,
            subscribers: HashMap::new(),
            cards: all(),
        }
    }
}

#[derive(Clone)]
pub struct UnoSocket {
    pub session: actix_ws::Session,
}

impl Debug for UnoSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("")
    }
}

pub fn room_and_player(req: HttpRequest, session: Session) -> (u32, u32) {
    let room_id = req
        .match_info()
        .get("room_id")
        .unwrap()
        .parse::<u32>()
        .unwrap();
    let player_id = session.get::<u32>("player_id").unwrap().unwrap_or(0);
    (room_id, player_id)
}
