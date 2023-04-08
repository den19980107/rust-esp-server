mod api;
mod mqtt;
mod wifi;

use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use anyhow::Result;
use dht11::Dht11;
use dotenvy::dotenv;
#[macro_use]
extern crate dotenv_codegen;

use embedded_svc::{http::Method, mqtt::client::QoS};
use esp_idf_hal::{
    adc::{config::Config, AdcChannelDriver, AdcDriver, Atten11dB},
    delay::Ets,
    gpio::{Gpio33, PinDriver},
    prelude::Peripherals,
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, http::server::EspHttpServer};

use crate::{
    api::{ReadHumidityHandler, ReadTempHandler, TurnOffLedHandler, TurnOnLedHandler},
    mqtt::init_mqtt,
};

fn main() -> Result<()> {
    dotenv().ok();

    let wifi_ssid = dotenv!("WIFI_SSID");
    let wifi_pass = dotenv!("WIFI_PASS");

    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals: Peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let led_pin = Arc::new(Mutex::new(
        PinDriver::output(peripherals.pins.gpio2).unwrap(),
    ));

    // status indication of esp32 is booting succesfully
    led_pin.lock().unwrap().set_high().unwrap();

    let _wifi = wifi::wifi(peripherals.modem, sysloop.clone(), wifi_ssid, wifi_pass)?;

    let mut server: EspHttpServer = EspHttpServer::new(&Default::default())?;

    let dht11_pin = PinDriver::input_output_od(peripherals.pins.gpio4).unwrap();
    let dht11 = Arc::new(Mutex::new(Dht11::new(dht11_pin)));
    let adc = Arc::new(Mutex::new(
        AdcDriver::new(peripherals.adc2, &Config::new().calibration(true)).unwrap(),
    ));
    let photoresistance_pin: Arc<
        Mutex<esp_idf_hal::adc::AdcChannelDriver<'_, Gpio33, Atten11dB<_>>>,
    > = Arc::new(Mutex::new(
        AdcChannelDriver::new(peripherals.pins.gpio33).unwrap(),
    ));

    server
        .handler(
            "/led/on",
            Method::Get,
            TurnOnLedHandler::new(led_pin.clone()),
        )
        .unwrap();

    server
        .handler(
            "/led/off",
            Method::Get,
            TurnOffLedHandler::new(led_pin.clone()),
        )
        .unwrap();

    server
        .handler("/temp", Method::Get, ReadTempHandler::new(dht11.clone()))
        .unwrap();

    server
        .handler(
            "/humidity",
            Method::Get,
            ReadHumidityHandler::new(dht11.clone()),
        )
        .unwrap();

    let mut client = init_mqtt();

    // if server succesfully boot up, turn off the led
    led_pin.lock().unwrap().set_low().unwrap();

    loop {
        println!("loop...");

        let mut delay = Ets;
        match dht11.lock().unwrap().perform_measurement(&mut delay) {
            Ok(measurement) => {
                let temp = measurement.temperature as f32 / 10.0;
                let humidity = measurement.humidity as f32 / 10.0;
                let json_message = format!(
                    r#"{{
                        "temp": {},
                        "humidity": {}
                    }}"#,
                    temp, humidity,
                );
                client.publish(
                    "worker/rawData",
                    QoS::AtMostOnce,
                    false,
                    json_message.as_bytes(),
                )?;
            }
            Err(err) => {
                println!("read temp from sensor failed, err = {:?}", err);
            }
        };

        match adc
            .lock()
            .unwrap()
            .read(&mut photoresistance_pin.lock().unwrap())
        {
            Ok(v) => {
                println!("adc: {}", v);
            }
            Err(err) => {
                println!("read adc value failed, {:?}", err);
            }
        }

        thread::sleep(Duration::from_secs(5));
    }
}
