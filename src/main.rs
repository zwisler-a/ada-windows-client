#![windows_subsystem = "windows"]

use std::time::Duration;

use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS, Transport};
use serde_json::{json};
use tokio::time::interval;
use tokio_rustls::rustls::ClientConfig;
use crate::tray_icon::Message;

mod command_handlers;
mod device_definition;
mod tray_icon;

static DEVICE_ID: &str = "15617999";
static USER_ID: &str = "1";


#[tokio::main(flavor = "current_thread")]
async fn main() {
    simple_logger::init().unwrap();

    log::debug!("Main started!");
    let mut tray_rx = tray_icon::define_tray_menu();

    log::debug!("Device json parsed");

    let mut mqttoptions = MqttOptions::new(
        "xxx",
        "xxx",
        1883,
    );
    mqttoptions.set_credentials("xxx", "xxx");
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let mut root_cert_store = tokio_rustls::rustls::RootCertStore::empty();
    root_cert_store.add_parsable_certificates(
        rustls_native_certs::load_native_certs().expect("could not load platform certs"),
    );

    let client_config = ClientConfig::builder()
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    mqttoptions.set_transport(Transport::tls_with_config(client_config.into()));

    let mut device_info = device_definition::get_device_info();
    device_info["id"] = json!(DEVICE_ID);
    device_info["name"]["name"] = json!("PC");

    let announcement = json!({
        "deviceId": DEVICE_ID,
        "userId": USER_ID,
        "device": device_info
    });

    log::debug!("Connecting to MQTT");
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    log::debug!("Publish discover message");
    client.subscribe(format!("/ada/{USER_ID}/{DEVICE_ID}/action"), QoS::ExactlyOnce).await.unwrap();
    client.publish("/ada/discover", QoS::AtLeastOnce, true, announcement.to_string()).await.unwrap();


    log::debug!("Starting handler loop...");
    let mut ticker = interval(Duration::from_secs(5));
    loop {
        tokio::select! {
            // Periodic updates
            _ = ticker.tick() => {
                unsafe {
                    command_handlers::send_status(&client).await;
                }
            },

            // Handle tray
            Some(message) = tray_rx.recv() => {
                match message {
                    Message::Quit => {
                        println!("Quit");
                        break; // Exit the loop when Quit is received
                    }
                }
            }

            // Handle MQTT events
            event = eventloop.poll() => match event {
                Ok(Event::Incoming(Incoming::Publish(p))) => unsafe {
                    log::debug!("Topic: {}, Payload: {:?}", p.topic, p.payload);
                    let data: serde_json::Value = serde_json::from_slice(&*p.payload).unwrap();
                    let command = data["command"].as_str().unwrap();

                    match command {
                        "action.devices.commands.setVolume" => command_handlers::set_volume(&data, &client).await,
                        "action.devices.commands.mute" => command_handlers::mute(&data, &client).await,
                        "action.devices.commands.volumeRelative" => command_handlers::set_volume_relative(data, &client).await,
                        "action.devices.commands.mediaNext" => command_handlers::next_track(data, &client).await,
                        "action.devices.commands.mediaPrevious" => command_handlers::previous_track(data, &client).await,
                        "action.devices.commands.mediaPause" => command_handlers::pause(data, &client).await,
                        "action.devices.commands.mediaStop" => command_handlers::pause(data, &client).await,
                        "action.devices.commands.mediaResume" => command_handlers::unpause(data, &client).await,
                        "action.devices.commands.OnOff" => command_handlers::on_off(data, &client).await,
                        _ => {}
                    }
                }
                Ok(Event::Incoming(_i)) => {}
                Ok(Event::Outgoing(_o)) => {}
                Err(_e) => {                }
            }
        }
    }
}






