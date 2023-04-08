use std::thread;

use embedded_svc::{
    mqtt::client::{Connection, MessageImpl},
    utils::mqtt::client::ConnState,
};
use esp_idf_svc::mqtt::client::{EspMqttClient, MqttClientConfiguration};
use esp_idf_sys::EspError;

pub fn init_mqtt() -> EspMqttClient<ConnState<MessageImpl, EspError>> {
    let conf = MqttClientConfiguration {
        client_id: Some("esp32"),
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),

        ..Default::default()
    };

    let (client, mut connection) =
        EspMqttClient::new_with_conn("mqtt://192.168.0.20:1883", &conf).unwrap();
    println!("MQTT client started");

    thread::spawn(move || {
        println!("MQTT Listening for messages");

        while let Some(msg) = connection.next() {
            match msg {
                Err(e) => println!("MQTT Message ERROR: {}", e),
                Ok(msg) => println!("MQTT Message: {:?}", msg),
            }
        }

        println!("MQTT connection loop exit");
    });

    return client;
}
