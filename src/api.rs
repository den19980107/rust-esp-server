use std::sync::{Arc, Mutex};

use dht11::Dht11;
use embedded_svc::{
    http::server::{Handler, Request},
    io::Write,
};
use esp_idf_hal::{
    delay::Ets,
    gpio::{Gpio2, Gpio4, InputOutput, Output, PinDriver},
};
use esp_idf_svc::http::server::EspHttpConnection;

pub struct TurnOnLedHandler<'a> {
    led_pin: Arc<Mutex<PinDriver<'a, Gpio2, Output>>>,
}

impl TurnOnLedHandler<'static> {
    pub fn new(led_pin: Arc<Mutex<PinDriver<Gpio2, Output>>>) -> TurnOnLedHandler {
        TurnOnLedHandler { led_pin }
    }
}

impl Handler<EspHttpConnection<'_>> for TurnOnLedHandler<'static> {
    fn handle(
        &self,
        connection: &mut EspHttpConnection<'_>,
    ) -> embedded_svc::http::server::HandlerResult {
        self.led_pin.lock().unwrap().set_high().unwrap();
        let req = Request::wrap(connection);
        req.into_ok_response()?.write_all("success".as_bytes())?;
        Ok(())
    }
}

pub struct TurnOffLedHandler<'a> {
    led_pin: Arc<Mutex<PinDriver<'a, Gpio2, Output>>>,
}

impl TurnOffLedHandler<'static> {
    pub fn new(led_pin: Arc<Mutex<PinDriver<Gpio2, Output>>>) -> TurnOffLedHandler {
        TurnOffLedHandler { led_pin }
    }
}

impl Handler<EspHttpConnection<'_>> for TurnOffLedHandler<'static> {
    fn handle(
        &self,
        connection: &mut EspHttpConnection<'_>,
    ) -> embedded_svc::http::server::HandlerResult {
        self.led_pin.lock().unwrap().set_low().unwrap();
        let req = Request::wrap(connection);
        req.into_ok_response()?.write_all("success".as_bytes())?;
        Ok(())
    }
}

pub struct ReadTempHandler<'a> {
    dht11: Arc<Mutex<Dht11<PinDriver<'a, Gpio4, InputOutput>>>>,
}

impl ReadTempHandler<'static> {
    pub fn new(dht11: Arc<Mutex<Dht11<PinDriver<Gpio4, InputOutput>>>>) -> ReadTempHandler {
        ReadTempHandler { dht11 }
    }
}

impl Handler<EspHttpConnection<'_>> for ReadTempHandler<'static> {
    fn handle(
        &self,
        connection: &mut EspHttpConnection<'_>,
    ) -> embedded_svc::http::server::HandlerResult {
        let mut delay = Ets;
        let req = Request::wrap(connection);
        match self.dht11.lock().unwrap().perform_measurement(&mut delay) {
            Ok(measurement) => {
                let temp = measurement.temperature as f32 / 10.0;
                req.into_ok_response()?
                    .write_all(format!("{}", temp).as_bytes())?;
            }
            Err(err) => {
                println!("read temp from sensor failed, err = {:?}", err);
                req.into_status_response(500)?.write_all(
                    format!("read temp from sesor failed, err = {:?}", err).as_bytes(),
                )?;
            }
        };
        Ok(())
    }
}

pub struct ReadHumidityHandler<'a> {
    dht11: Arc<Mutex<Dht11<PinDriver<'a, Gpio4, InputOutput>>>>,
}

impl ReadHumidityHandler<'static> {
    pub fn new(dht11: Arc<Mutex<Dht11<PinDriver<Gpio4, InputOutput>>>>) -> ReadHumidityHandler {
        ReadHumidityHandler { dht11 }
    }
}

impl Handler<EspHttpConnection<'_>> for ReadHumidityHandler<'static> {
    fn handle(
        &self,
        connection: &mut EspHttpConnection<'_>,
    ) -> embedded_svc::http::server::HandlerResult {
        let mut delay = Ets;
        let req = Request::wrap(connection);
        match self.dht11.lock().unwrap().perform_measurement(&mut delay) {
            Ok(measurement) => {
                let humidity = measurement.humidity as f32 / 10.0;
                req.into_ok_response()?
                    .write_all(format!("{}", humidity).as_bytes())?;
            }
            Err(err) => {
                println!("read humidity from sensor failed, err = {:?}", err);
                req.into_status_response(500)?.write_all(
                    format!("read humidity from sesor failed, err = {:?}", err).as_bytes(),
                )?;
            }
        };
        Ok(())
    }
}
