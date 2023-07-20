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
                                        let (tx, rx): (Sender<Location>, Receiver<Location>) =
                                            mpsc::channel();

                                        // startup an interval
                                        ctx.run_interval(
                                            Duration::from_millis(10),
                                            move |act, ctx| {
                                                for location in rx.try_iter() {
                                                    let data = location.serialize().unwrap();
                                                    let message = Message {
                                                        message_type: MessageType::UserLocation,
                                                        data,
                                                    };
                                                    ctx.binary(message.serialize().unwrap());
                                                }
                                            },
                                        );

                                        // insert the sender into the pool
                                        // other thread/sockets will be sending through this
                                        pool.insert(auth.id, Mutex::new(tx.clone()));
                                    }
                                    Err(_) => {
                                        return; // do nothing
                                    }
                                }
                                return;
                            }
                            MessageType::UserLocation => {
                                return; // do nothing
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
                                            let _ = s_lock.send(location);
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
            Ok(ws::Message::Close(rsn)) => println!("closing: {:?}", rsn),
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
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

type MessagingPools = RwLock<HashMap<String, RwLock<HashMap<u64, Mutex<Sender<Location>>>>>>;
