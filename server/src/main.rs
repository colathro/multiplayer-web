use actix::{Actor, AsyncContext, StreamHandler};
use actix_web::{
    web::{self, Data},
    App, Error, HttpRequest, HttpResponse, HttpServer,
};
use actix_web_actors::ws;
use shared::*;
use std::sync::RwLock;
use std::sync::{mpsc, Mutex};
use std::{cell::RefCell, collections::HashMap};
use std::{
    sync::mpsc::{Receiver, Sender},
    time::Duration,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    return start().await;
}

/// Define HTTP actor
struct MyWs {
    user_id: RefCell<Option<u64>>,
    url: RefCell<Option<String>>,
    messaging_pools: Data<MessagingPools>,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Binary(bin)) => {
                let message_res = Message::deserialize(&bin);
                match message_res {
                    Ok(message) => {
                        match message.message_type {
                            MessageType::Auth => {
                                // register url and set user_id
                                match Auth::deserialize(&message.data) {
                                    Ok(auth) => {
                                        if self.user_id.borrow().is_some()
                                            || self.url.borrow().is_some()
                                        {
                                            return; // we are already authed, do nothing
                                        }

                                        *self.user_id.borrow_mut() = Some(auth.id);
                                        *self.url.borrow_mut() = Some(auth.url.clone());

                                        let mut pools = self.messaging_pools.write().unwrap();

                                        if pools.get(&auth.url).is_none() {
                                            // construct pool for url
                                            pools.insert(
                                                auth.url.clone(),
                                                RwLock::new(HashMap::new()),
                                            );
                                        }

                                        let mut pool =
                                            pools.get(&auth.url).unwrap().write().unwrap();
                                        let (tx, rx): (Sender<PoolMessage>, Receiver<PoolMessage>) =
                                            mpsc::channel();

                                        // startup an interval
                                        ctx.run_interval(
                                            Duration::from_millis(10),
                                            move |_, ctx| {
                                                for pool_message in rx.try_iter() {
                                                    match pool_message {
                                                        PoolMessage::LocationUpdate(location) => {
                                                            let data =
                                                                location.serialize().unwrap();
                                                            let message = Message {
                                                                message_type:
                                                                    MessageType::UserLocation,
                                                                data,
                                                            };
                                                            ctx.binary(
                                                                message.serialize().unwrap(),
                                                            );
                                                        }
                                                        PoolMessage::SpawnEntity(spawn) => {
                                                            let data = spawn.serialize().unwrap();
                                                            let message = Message {
                                                                message_type: MessageType::Spawn,
                                                                data,
                                                            };
                                                            ctx.binary(
                                                                message.serialize().unwrap(),
                                                            );
                                                        }
                                                        PoolMessage::DespawnEntity(despawn) => {
                                                            let data = despawn.serialize().unwrap();
                                                            let message = Message {
                                                                message_type: MessageType::Despawn,
                                                                data,
                                                            };
                                                            ctx.binary(
                                                                message.serialize().unwrap(),
                                                            );
                                                        }
                                                    }
                                                }
                                            },
                                        );

                                        // insert the sender into the pool
                                        // other thread/sockets will be sending through this
                                        pool.insert(auth.id, Mutex::new(tx.clone()));

                                        for (id, sender) in pool.iter() {
                                            if *id == auth.id {
                                                continue;
                                            }
                                            let s_lock = sender.lock().unwrap();
                                            let _ = s_lock.send(PoolMessage::SpawnEntity(Spawn {
                                                id: auth.id,
                                                icon: auth.url.clone(),
                                            }));
                                        }

                                        for (id, _) in pool.iter() {
                                            if *id == auth.id {
                                                continue;
                                            }
                                            let _ =
                                                tx.clone().send(PoolMessage::SpawnEntity(Spawn {
                                                    id: *id,
                                                    icon: String::from("test"),
                                                }));
                                        }
                                    }
                                    Err(_) => {
                                        return; // do nothing
                                    }
                                }
                                return;
                            }
                            MessageType::UserLocation => {
                                return; // server doesn't handle these events incoming
                            }
                            MessageType::Spawn => {
                                return; // server doesn't handle these events incoming
                            }
                            MessageType::Despawn => {
                                return; // server doesn't handle these events incoming
                            }
                            MessageType::MyLocation => match Location::deserialize(&message.data) {
                                Ok(mut location) => {
                                    if let (Some(url), Some(user_id)) =
                                        (self.url.borrow().as_ref(), self.user_id.borrow().as_ref())
                                    {
                                        location.id = *user_id;
                                        let pools = self.messaging_pools.read().unwrap();
                                        let pool = pools.get(url).unwrap().read().unwrap();
                                        for (_, sender) in pool.iter() {
                                            let s_lock = sender.lock().unwrap();
                                            let _ =
                                                s_lock.send(PoolMessage::LocationUpdate(location));
                                        }
                                    }
                                }
                                Err(_) => return,
                            },
                        }
                    }
                    Err(_) => {
                        return; // do nothing
                    }
                }
            }
            Ok(ws::Message::Close(rsn)) => {
                println!("closing: {:?}", rsn);

                if let (Some(user_id), Some(url)) =
                    (self.user_id.borrow().as_ref(), self.url.borrow().as_ref())
                {
                    let pools = self.messaging_pools.read().unwrap();
                    let mut pool = pools.get(url).unwrap().write().unwrap();
                    pool.remove(user_id);
                }

                if let (Some(user_id), Some(url)) =
                    (self.user_id.borrow().as_ref(), self.url.borrow().as_ref())
                {
                    let pools = self.messaging_pools.read().unwrap();
                    let pool = pools.get(url).unwrap().read().unwrap();
                    for (_, sender) in pool.iter() {
                        let s_lock = sender.lock().unwrap();
                        let _ = s_lock.send(PoolMessage::DespawnEntity(Despawn { id: *user_id }));
                    }
                }
            }
            _ => (),
        }
    }
}

async fn index(
    req: HttpRequest,
    stream: web::Payload,
    messaging_pools: Data<MessagingPools>,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        MyWs {
            user_id: RefCell::new(None),
            url: RefCell::new(None),
            messaging_pools,
        },
        &req,
        stream,
    );

    println!("opening connection");

    resp
}

pub async fn start() -> std::io::Result<()> {
    let map: MessagingPools = RwLock::new(HashMap::new());
    let data = Data::new(map);

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/ws/", web::get().to(index))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

type MessagingPools = RwLock<HashMap<String, RwLock<HashMap<u64, Mutex<Sender<PoolMessage>>>>>>;

enum PoolMessage {
    LocationUpdate(Location),
    SpawnEntity(Spawn),
    DespawnEntity(Despawn),
}
