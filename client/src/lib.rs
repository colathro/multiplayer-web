use rand::prelude::*;
use shared::{Auth, Despawn, Location, Message, MessageType, Spawn};
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct WebSocketSend {
    websocket: WebSocket,
}

#[wasm_bindgen]
pub unsafe fn send_location(wss: &mut WebSocketSend, x: f32, y: f32) {
    let location = Location { id: 123, x, y };
    let message = Message {
        message_type: MessageType::MyLocation,
        data: location.serialize().unwrap(),
    };

    if let Ok(data) = message.serialize() {
        let res = wss.websocket.send_with_u8_array(&data);
        if let Err(e) = res {
            console_log!("{:?}", e);
        }
    }
}

#[wasm_bindgen]
pub unsafe fn start_websocket(
    set_cursor_callback: js_sys::Function,
    spawn_cursor_callback: js_sys::Function,
    despawn_cursor_callback: js_sys::Function,
    url: js_sys::JsString,
) -> Result<WebSocketSend, JsValue> {
    // Connect to an echo server
    let ws: WebSocket = WebSocket::new("wss://multiplayer-web.fly.dev/ws/")?;
    // For small binary messages, like CBOR, Arraybuffer is more efficient than Blob handling
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        // Handle difference Text/Binary,...
        if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
            let array = js_sys::Uint8Array::new(&abuf);

            match Message::deserialize(&array.to_vec()) {
                Ok(message) => match message.message_type {
                    MessageType::UserLocation => {
                        let location = Location::deserialize(&message.data).unwrap();
                        let _ = set_cursor_callback.call3(
                            &JsValue::null(),
                            &JsValue::from(location.id),
                            &JsValue::from(location.x),
                            &JsValue::from(location.y),
                        );
                    }
                    MessageType::Spawn => {
                        let spawn = Spawn::deserialize(&message.data).unwrap();
                        let _ = spawn_cursor_callback.call2(
                            &JsValue::null(),
                            &JsValue::from(spawn.id),
                            &JsValue::from(spawn.icon),
                        );
                    }
                    MessageType::Despawn => {
                        let despawn = Despawn::deserialize(&message.data).unwrap();
                        let _ = despawn_cursor_callback
                            .call1(&JsValue::null(), &JsValue::from(despawn.id));
                    }
                    _ => return,
                },
                Err(e) => {
                    console_log!("{:?} - Failed to deserialize message.", e);
                }
            }
        } else {
            console_log!("message event, received Unknown: {:?}", e.data());
        }
    });
    // set message event handler on WebSocket
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // forget the callback to keep it alive
    onmessage_callback.forget();

    let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
        console_log!("error event: {:?}", e);
    });
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    let ws_clone = ws.clone();
    let onopen_callback = Closure::<dyn FnMut()>::new(move || {
        console_log!("socket opened");

        let mut rng = rand::thread_rng();

        let location = Auth {
            id: rng.gen_range(u64::MIN..u64::MAX),
            url: String::from(&url),
        };

        let message = Message {
            message_type: MessageType::Auth,
            data: location.serialize().unwrap(),
        };

        if let Ok(data) = message.serialize() {
            let res = ws_clone.send_with_u8_array(&data);
            if let Err(e) = res {
                console_log!("{:?}", e);
            }
        }
    });

    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();
    let ws_clone = ws.clone();

    Ok(WebSocketSend {
        websocket: ws_clone,
    })
}
