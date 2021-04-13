use crate::user::User;
use serde::Serialize;
use crate::card::{Card, CardColor, GameState};
use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Clone, Serialize, Debug)]
pub struct Room {
    pub players: Vec<User>,
    pub active: bool,
    pub deck: Vec<(u8, Card)>,
    pub board: Vec<(u8, Card)>,
    pub current_player: User,
    turn: u8,
    pub color: CardColor,
    pub state: Option<GameState>,
    pub chain_count: u8,
    pub direction: bool
}

impl Room {
    pub fn new() -> Room {
        Room {
            players: vec![],
            active: false,
            deck: vec![],
            board: vec![],
            current_player: User::new("".to_string(), 0),
            turn: 0,
            color: CardColor::Red,
            state: None,
            chain_count: 0,
            direction: false
        }
    }

    pub fn push(&mut self, user: User) -> Room {
        self.players.push(user);
        self.clone()
    }

    pub fn player(&mut self, player_id: u32) -> &mut User {
        self.players.iter_mut().find(|player| player.id == player_id).unwrap()
    }

    pub fn update_player(&mut self) {
        self.current_player = self.players[self.turn as usize].clone();
    }

    pub fn to_center(&mut self) {
        self.board.push(self.deck.pop().unwrap());
    }

    pub fn draw(&mut self) -> (u8, Card) {
        if self.deck.len() < 2 {
            self.deck = vec![self.board.clone(), self.deck.clone()].concat();
            let mut rng = thread_rng();
            self.deck.shuffle(&mut rng);
        }
    
        let result = self.deck[0].clone();
        self.deck = self.deck[1..].to_vec();
        result
    }

    pub fn top(&self) -> (u8, Card) {
        self.board[self.board.len() - 1].clone()
    }

    pub fn left(&mut self) {
        if self.turn == 0 {
            self.turn = self.players.len() as u8 - 1;
        } else {
            self.turn -= 1;
        }
    }

    pub fn right(&mut self) {
        if self.turn == self.players.len() as u8 - 1 {
            self.turn = 0;
        } else {
            self.turn += 1;
        }
    }

    pub fn next(&mut self) {
        if self.direction {
            self.right();
        } else {
            self.left();
        }
    
        let player = &self.players[self.turn as usize];
    
        if player.hand.len() == 0 {
            self.next();
        }
    }
}
