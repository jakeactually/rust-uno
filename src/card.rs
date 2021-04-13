use std::fmt::Display;
use serde::{Serialize, Deserialize};

#[derive(PartialEq, Eq, Clone, Debug, Serialize, Deserialize)]
pub enum CardColor {
    Red,
    Green,
    Blue,
    Yellow
}

impl Display for CardColor {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::result::Result<(), ::std::fmt::Error> {
        match *self {
            CardColor::Red => f.write_str("red"),
            CardColor::Green => f.write_str("green"),
            CardColor::Blue => f.write_str("blue"),
            CardColor::Yellow => f.write_str("yellow"),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug, Serialize)]
pub enum Card {
    Number(u8, CardColor),
    Stop(CardColor),
    Reverse(CardColor),
    Plus2(CardColor),
    ChangeColor,
    Plus4
}

#[derive(Clone, Serialize, Debug, PartialEq, Eq)]
pub enum GameState {
    Stop,
    Plus2,
    Plus4
}

impl Card {
    pub fn is_stop(&self) -> bool {
        match self {
            Card::Stop(_) => true,
            _ => false
        }
    }

    pub fn is_reverse(&self) -> bool {
        match self {
            Card::Reverse(_) => true,
            _ => false
        }
    }

    pub fn is_plus_2(&self) -> bool {
        match self {
            Card::Plus2(_) => true,
            _ => false
        }
    }

    pub fn is_normal(&self) -> bool {
        match self {
            Card::Number(_, _) => true,
            _ => false
        }
    }

    pub fn is_color_card(&self) -> bool {
        *self == Card::Plus4 || *self == Card::ChangeColor
    }

    pub fn can_chain(&self) -> bool {
        *self == Card::Plus4 || self.is_plus_2() || self.is_stop()
    }

    pub fn to_game_state(&self) -> Option<GameState> {
        match self {
            Card::Stop(_) => Some(GameState::Stop),
            Card::Plus2(_) => Some(GameState::Plus2),
            Card::Plus4 => Some(GameState::Plus4),
            _ => None,
        }
    }

    pub fn get_color(self) -> Option<CardColor> {
        match self {
            Card::Number(_, color) => Some(color),
            Card::Stop(color) => Some(color),
            Card::Reverse(color) => Some(color),
            Card::Plus2(color) => Some(color),
            _ => None,
        }
    }

    pub fn matches_color(self, color: CardColor) -> bool {
        self.get_color().iter().all(|c| *c == color)
    }

    pub fn matches(self, game_state: Option<GameState>, choosen_color: CardColor, other: Card) -> Option<String> {
        match game_state {
            Some(GameState::Stop) => make_error(other.is_stop(), "You can only chain or pass"),
            Some(GameState::Plus2) => make_error(other.is_plus_2(), "You can only chain or pass"),
            Some(GameState::Plus4) => make_error(other == Card::Plus4, "You can only chain or pass"),
            None => self.free_match(choosen_color, other)
        }
    }

    pub fn free_match(self, choosen_color: CardColor, other: Card) -> Option<String> {
        match self {
            Card::Number(number, color) => match other {
                Card::Number(number2, color2) =>
                    make_error(number == number2 || color == color2, "Invalid Move"),
                _ => make_error(other.matches_color(color), "Wrong color")
            },            
            Card::Stop(color) => make_error(other.is_stop() || other.matches_color(color), "Wrong color"),
            Card::Reverse(color) => make_error(other.is_reverse() || other.matches_color(color), "Wrong color"),
            Card::Plus2(color) => make_error(other.is_plus_2() || other.matches_color(color), "Wrong color"),
            Card::ChangeColor => make_error(other.matches_color(choosen_color.clone()), format!("Choosen color is {}", choosen_color).as_str()),
            Card::Plus4 => make_error(other.matches_color(choosen_color.clone()), format!("Choosen color is {}", choosen_color).as_str()),
        }
    }
}

pub fn all() -> Vec<Card> {
    let colors = vec![CardColor::Red, CardColor::Green, CardColor::Blue, CardColor::Yellow];

    vec![
        colors.iter().map(|color| Card::Number(0, color.clone())).collect(),
        (1..9).flat_map(|n| colors.iter().map(move |c| Card::Number(n, c.clone()))).collect(),
        colors.iter().map(|color| Card::Stop(color.clone())).collect(),
        colors.iter().map(|color| Card::Reverse(color.clone())).collect(),
        colors.iter().map(|color| Card::Plus2(color.clone())).collect(),
        (0..3).map(|_| Card::ChangeColor).collect::<Vec<Card>>(),
        (0..3).map(|_| Card::Plus4).collect::<Vec<Card>>(),
    ].concat()
}

fn make_error(condition: bool, message: &str) -> Option<String> {
    if condition { None } else { Some(message.to_string()) }
}
