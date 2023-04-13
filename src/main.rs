extern crate log;
use log::{info, debug};

use dittolive_ditto::{prelude::*};
use dotenv::dotenv;
use rust_gpiozero::*;
use serde::{Deserialize, Serialize};
use serde_json::{json};
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;

#[derive(Serialize, Deserialize, Debug)]
struct State {
    state: bool,
    _id: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();
    debug!("Blink the Light!");
    let mut button = Button::new(17);
    let led = LED::new(23);
    led.off();

    let (sender, receiver) = channel::<(Vec<BoxedDocument>, LiveQueryEvent)>();
    let event_handler = move |documents: Vec<BoxedDocument>, event: LiveQueryEvent| {
        sender.send((documents, event)).unwrap();
    };

    let ditto = Ditto::builder()
    .with_root(Arc::new(PersistentRoot::from_current_exe().unwrap()))
        .with_identity(|ditto_root| {
            let app_id = AppId::from_str("09fcd60d-69d2-414d-bc66-9c2475077258").unwrap();
            identity::OfflinePlayground::new(ditto_root, app_id)
        })?
        .with_minimum_log_level(LogLevel::Info)
        .with_transport_config(|_identity| -> TransportConfig {
            let mut transport_config = TransportConfig::new();
            transport_config.peer_to_peer.bluetooth_le.enabled = true;
            transport_config.peer_to_peer.lan.enabled = false;
            transport_config
        })?
        .build()?;
    
    ditto.set_license_from_env("DITTO_LICENSE")?;
    ditto.start_sync()?;

    let store = ditto.store();
    let collection = store.collection("button_state").unwrap();
    let _sub = collection.find_all().subscribe();

    let _lq: LiveQuery = collection.find_all().observe_local(event_handler)?;
    thread::spawn(move || {
        loop {
            button.wait_for_press(None);
            debug!("Button Pressed");
            let id = DocumentId::new(&"77".to_string()).unwrap();
            let find_res = collection.find_by_id(id).exec();
            let state_doc = match find_res {
                Ok(doc) => doc.typed().unwrap(),
                Err(_error) => State{_id: "77".to_string(), state: false},
            };
            debug!("DOC STATE {:?}", state_doc);
            if state_doc.state {
                let _res = collection.upsert(json!({"_id": "77", "state": false}));
            } else {
                let _res = collection.upsert(json!({"_id": "77", "state": true}));
            }      
        }
    });

    loop {
        let (documents, event) = receiver.recv().unwrap();
        info!("We have an event {:?}", event);
        for doc in documents {
            let v: State = doc.typed()?;
            info!("\tDocument State {:?}", v.state);
            if v._id == "77" {
                if v.state {
                    led.on();
                } else {
                    led.off();
                }
            }
        }
    }         
}
