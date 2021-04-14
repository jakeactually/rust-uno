mod user;
mod room;
mod uno;
mod message;
mod game;
mod card;

use std::sync::{Mutex, MutexGuard};
use actix_web::{web, App, HttpRequest, HttpServer, Responder};
use actix_session::{Session, CookieSession};
use user::User;
use room::Room;
use message::{UnoSubscription, UnoMessage};
use uno::{Uno, room_and_player};
use serde::{Serialize, Deserialize};
use actix_web_actors::ws;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let data = web::Data::new(Mutex::new(Uno::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(CookieSession::signed(&[0; 32]).secure(false))
            .route("/api/", web::get().to(index))
            .route("/api/new-room", web::post().to(new_room))
            .route("/api/room/{room_id}", web::get().to(room))
            .route("/api/state/{room_id}", web::get().to(state))
            .route("/api/join-room/{room_id}", web::post().to(join_room))
            .route("/api/play/{room_id}", web::get().to(game::play))
            .route("/api/turn/{room_id}", web::post().to(game::turn))
            .route("/api/draw/{room_id}", web::post().to(game::draw))
            .route("/api/penalty/{room_id}", web::post().to(game::penalty))
            .route("/api/pass/{room_id}", web::post().to(game::pass))
    })
    .bind(("0.0.0.0", 8080))? 
    .run()
    .await
}

async fn index(data: web::Data<Mutex<Uno>>, session: Session) -> impl Responder {
    let player_id = session.get::<u32>("player_id").unwrap().unwrap_or(0);    
    let users = &data.lock().unwrap().users;
    let default = User::new("".to_string(), 0);
    let user = users.get(&player_id).unwrap_or(&default);
    web::Json(user.clone())
}

#[derive(Deserialize)]
struct NewRoomReq {
    pub username: String
}

#[derive(Serialize)]
struct NewRoomRes {
    pub room_id: u32
}

async fn new_room(data: web::Data<Mutex<Uno>>, session: Session, form: web::Json<NewRoomReq>) -> impl Responder {
    let mut context = data.lock().unwrap();
    context.room_index += 1;
    let user = get_user(&mut context, form, session);
    let room_index = context.room_index;
    context.rooms.insert(room_index, Room::new().push(user));
    web::Json(NewRoomRes { room_id: context.room_index })
}

fn get_user(context: &mut MutexGuard<Uno>, form: web::Json<NewRoomReq>, session: Session) -> User {    
    let mut player_id = session.get::<u32>("player_id").unwrap().unwrap_or(0);

    if player_id == 0 {
        context.user_index += 1;
        player_id = context.user_index;
        session.set("player_id", player_id).unwrap();
    }

    if context.users.get(&player_id).is_none() {
        context.users.insert(player_id, User::new(form.username.clone(), player_id));
    }

    context.users.get(&player_id).unwrap().clone()
}

async fn room(data: web::Data<Mutex<Uno>>, req: HttpRequest, session: Session) -> impl Responder {
    let context = data.lock().unwrap();
    let (room_id, player_id) = room_and_player(req, session);

    let room = context.rooms.get(&room_id).unwrap();
    if !room.players.iter().any(|player| player.id == player_id) {
        return Err(actix_web::error::ErrorUnauthorized("Player not in room"))
    }

    Ok(web::Json(room.clone()))
}

async fn join_room(data: web::Data<Mutex<Uno>>, req: HttpRequest, session: Session, form: web::Json<NewRoomReq>) -> impl Responder {
    let mut context = data.lock().unwrap();
    let room_id = req.match_info().get("room_id").unwrap().parse::<u32>().unwrap();

    let user = get_user(&mut context, form, session);
    let room = context.rooms.get_mut(&room_id).unwrap();
    room.players.push(user);

    let subscribers = context.subscribers.get_mut(&room_id).unwrap();
    for subscription in subscribers.iter() {
        subscription.send(UnoMessage).await.unwrap().unwrap();
    }

    web::Json(NewRoomRes { room_id: context.room_index })
}

async fn state(data: web::Data<Mutex<Uno>>, req: HttpRequest, stream: web::Payload) -> impl Responder {
    let room_id = req.match_info().get("room_id").unwrap().parse::<u32>().unwrap();

    ws::start_with_addr(UnoSubscription {}, &req, stream).map(|(addr, resp)| {
        let subscribers = &mut data.lock().unwrap().subscribers;

        if !subscribers.contains_key(&room_id) {
            subscribers.insert(room_id, vec![]);
        }

        subscribers.get_mut(&room_id).unwrap().push(addr);
        resp
    })
}
