use crate::card::{Card, CardColor, GameState};
use crate::room::Room;
use crate::uno::{room_and_player, Uno};
use crate::user::User;
use actix_session::Session;
use actix_web::{web, HttpRequest, Responder};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, MutexGuard};

#[derive(Serialize)]
struct Game {
    pub room: Room,
    pub player: User,
}

pub async fn play(
    data: web::Data<Mutex<Uno>>,
    req: HttpRequest,
    session: Session,
) -> impl Responder {
    let mut context = data.lock().unwrap();
    let all = context.cards.clone();
    let (room_id, player_id) = room_and_player(req, session);
    let maybe_room = context.rooms.get_mut(&room_id);

    if maybe_room.is_none() {
        return Err(actix_web::error::ErrorNotFound("Room does not exist"));
    }
    
    let room = maybe_room.unwrap();
    let players_amount = room.players.len();

    if players_amount < 2 {
        return Err(actix_web::error::ErrorBadRequest("Not enough players"));
    }

    if !room.active {
        room.deck = all
            .iter()
            .enumerate()
            .map(|(i, c)| (i as u8, c.clone()))
            .collect();
        let mut rng = thread_rng();
        room.deck.shuffle(&mut rng);

        for player in room.players.iter_mut() {
            let (hand, new_deck) = (room.deck[0..7].to_vec(), room.deck[8..].to_vec());
            room.deck = new_deck;
            player.hand = hand;
        }

        room.to_center();

        while !room.top().1.is_normal() {
            room.to_center();
        }

        room.active = true;
    }

    room.update_player();
    let player = room.player(player_id).clone();

    Ok(web::Json(Game {
        room: room.clone(),
        player,
    }))
}

#[derive(Deserialize, Clone)]
pub struct TurnReq {
    pub card_id: u8,
    pub color: Option<CardColor>,
}

#[derive(Serialize)]
pub struct TurnRes {
    pub room_id: u32,
}

pub async fn turn(
    data: web::Data<Mutex<Uno>>,
    req: HttpRequest,
    session: Session,
    form: web::Json<TurnReq>,
) -> impl Responder {
    let mut context = data.lock().unwrap();
    let all = context.cards.clone();
    let (room_id, player_id) = room_and_player(req, session);
    let room = context.rooms.get_mut(&room_id).unwrap();

    if room.current_player.id != player_id {
        return Err(actix_web::error::ErrorUnauthorized("Not your turn"));
    }

    let card_1 = room.top();
    let card_2 = all[form.card_id as usize].clone();

    let choosen_color = if card_1.1.is_color_card() {
        room.color.clone()
    } else {
        CardColor::Red
    };

    match card_1
        .1
        .matches(room.state.clone(), choosen_color, card_2.clone())
    {
        Some(error) => Err(actix_web::error::ErrorBadRequest(error)),
        None => {
            effects(room, (form.card_id, card_2.clone()), form.clone());
            do_turn(room, player_id, (form.card_id, card_2));
            notify(&mut context, room_id).await;
            Ok("")
        }
    }
}

async fn notify(context: &mut MutexGuard<'_, Uno>, room_id: u32) {
    let subscribers = context.subscribers.get_mut(&room_id).unwrap();
    for subscription in subscribers.iter_mut() {
        let _ = subscription.session.text("update").await;
    }
}

pub fn effects(room: &mut Room, card_tuple: (u8, Card), form: TurnReq) {
    if card_tuple.1.is_color_card() {
        room.color = form.color.unwrap();
    }

    if card_tuple.1.can_chain() {
        room.state = card_tuple.1.to_game_state();
        room.chain_count += 1;
    } else {
        room.chain_count = 0;
    }

    if card_tuple.1.is_reverse() {
        room.direction = !room.direction;
    }
}

pub fn do_turn(room: &mut Room, player_id: u32, card_tuple: (u8, Card)) {
    let player = room.player(player_id);
    player.hand = player
        .hand
        .clone()
        .into_iter()
        .filter(|(id, _)| *id != card_tuple.0)
        .collect();
    player.drawed = false;
    room.board.push(card_tuple);
    room.next();
}

pub async fn draw(
    data: web::Data<Mutex<Uno>>,
    req: HttpRequest,
    session: Session,
) -> impl Responder {
    let mut context = data.lock().unwrap();
    let (room_id, player_id) = room_and_player(req, session);
    let room = context.rooms.get_mut(&room_id).unwrap();
    let card = room.draw();
    let current_player_id = room.current_player.id;
    let player = room.player(player_id);

    if current_player_id != player_id {
        return Err(actix_web::error::ErrorUnauthorized("Not your turn"));
    }

    player.hand.push(card);
    player.drawed = true;
    notify(&mut context, room_id).await;

    Ok("")
}

pub async fn penalty(
    data: web::Data<Mutex<Uno>>,
    req: HttpRequest,
    session: Session,
) -> impl Responder {
    let mut context = data.lock().unwrap();
    let (room_id, player_id) = room_and_player(req, session);
    let room = context.rooms.get_mut(&room_id).unwrap();
    let game_state = (&room.state).clone().unwrap();

    if game_state == GameState::Plus2 {
        do_penalty(room, player_id, 2 * room.chain_count);
    } else if game_state == GameState::Plus4 {
        do_penalty(room, player_id, 4 * room.chain_count);
    } else if game_state == GameState::Stop {
        room.state = None;
        room.next();
    }

    notify(&mut context, room_id).await;

    ""
}

pub fn do_penalty(room: &mut Room, player_id: u32, amount: u8) {
    let mut cards = vec![];

    for _ in 0..amount {
        cards.push(room.draw());
    }

    let player = room.player(player_id);
    player.hand = vec![player.hand.clone(), cards].concat();
    room.state = None;
    room.chain_count = 0;
    room.next();
}

pub async fn pass(
    data: web::Data<Mutex<Uno>>,
    req: HttpRequest,
    session: Session,
) -> impl Responder {
    let mut context = data.lock().unwrap();
    let (room_id, player_id) = room_and_player(req, session);
    let room = context.rooms.get_mut(&room_id).unwrap();
    let player = room.player(player_id);

    if !player.drawed {
        return Err(actix_web::error::ErrorBadRequest("You must draw one card"));
    }

    player.drawed = false;
    room.next();
    notify(&mut context, room_id).await;

    Ok("")
}
