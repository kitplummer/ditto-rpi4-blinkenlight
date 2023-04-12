use rust_gpiozero::*;
use dittolive_ditto::{identity::*, prelude::*};

use std::str::FromStr;
use std::sync::Arc;

fn main() {
    let mut button = Button::new(17);
    let led = LED::new(23);
    led.off();
    let mut state = false;

    let ditto = Ditto::builder()
    .with_root(Arc::new(PersistentRoot::from_current_exe().unwrap()))
        .with_identity(|ditto_root| {
            let app_id = AppId::from_str("09fcd60d-69d2-414d-bc66-9c2475077258").unwrap();
            identity::OfflinePlayground::new(ditto_root, app_id)
        }).unwrap()
        .with_transport_config(|_identity| -> TransportConfig {
            let mut transport_config = TransportConfig::new();
            transport_config.peer_to_peer.bluetooth_le.enabled = true;
            transport_config.peer_to_peer.lan.enabled = true;
            transport_config
        }).unwrap()
        .build().unwrap();
    
    let shared_token = std::env::var("DITTO_LICENSE").expect("No License Env Var provided"); 
    ditto.set_offline_only_license_token(&shared_token).unwrap();
    ditto.start_sync();

    loop {
        println!("Hello, world!");
        button.wait_for_press(None);
        println!("pressed!!");
        if state {
            led.off();
            state = false;
        } else {
            led.on();
            state = true;
        }
    }            
}
