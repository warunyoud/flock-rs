use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ErrorEvent, MessageEvent, WebSocket};
use serde_json::json;

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub struct FlightClient {
    ws: WebSocket
}

#[wasm_bindgen]
impl FlightClient {
    pub fn new(base_url: &str, js_on_message: &js_sys::Function, js_on_open: &js_sys::Function, js_on_error: &js_sys::Function) -> FlightClient {
        let ws = WebSocket::new(base_url).unwrap();

        // let cloned_ws = ws.clone();
        let cloned_js_on_message = js_on_message.clone();
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                console_log!("message event, received Text: {:?}", txt);
                let this = JsValue::null();
                cloned_js_on_message.call1(&this, &txt).unwrap();
            } else {
                console_log!("message event, received Unknown: {:?}", e.data());
            }
        });

        // set message event handler on WebSocket
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        // forget the callback to keep it alive
        onmessage_callback.forget();

        let cloned_js_on_error = js_on_error.clone();
        let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
            console_log!("error event: {:?}", e);
            let this = JsValue::null();
            cloned_js_on_error.call1(&this, &e).unwrap(); 
        });
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();
    
        let cloned_js_on_open = js_on_open.clone();
        let onopen_callback = Closure::<dyn FnMut()>::new(move || {
            console_log!("socket opened");
            let this = JsValue::null();
            cloned_js_on_open.call0(&this).unwrap(); 
        });
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        FlightClient {
            ws
        }
    }

    pub fn subscribe(&self, topic: &str) -> Result<(), JsValue> {
        let message = json!({
            "topic": topic,
            "type": "Subscribe",
            "request_id": "subscribe_message",
        }).to_string();
        self.ws.send_with_str(&message)?;
        Ok(())
    }

    pub fn unsubscribe(&self, topic: &str) -> Result<(), JsValue> {
        let message = json!({
            "topic": topic,
            "type": "Unsubscribe",
            "request_id": "unsubscribe_message",
        }).to_string();
        self.ws.send_with_str(&message)?;
        Ok(())
    }

    pub fn ping(&self) -> Result<(), JsValue> {
        let message = json!({
            "type": "Ping",
        }).to_string();
        self.ws.send_with_str(&message)?;
        Ok(())
    }
}
