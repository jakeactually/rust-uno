#[derive(Debug, Clone)]
pub struct UnoSubscription;

use actix::{Actor, StreamHandler, Handler, Message};
use actix_web_actors::ws;

impl Actor for UnoSubscription {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for UnoSubscription {
    fn handle(&mut self, _: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {

    }
}

impl Handler<UnoMessage> for UnoSubscription {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, _: UnoMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text("update".to_string());
        Ok(true)
    }
}

#[derive(Message)]
#[rtype(result = "Result<bool, std::io::Error>")]
pub struct UnoMessage;
