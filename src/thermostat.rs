use std::fmt::Display;
use serde::Serialize;
use esp_idf_svc::hal::gpio::{Output, PinDriver, Gpio15};
use esp_idf_svc::mqtt::client::EspMqttClient;

#[derive(Debug)]
pub enum ThermostatEvent {
    MqttConnected,
    TopicSubscribed(u32),
    CommandReceived(ThermostatCommand),
    MessagePublished(u32)
}

pub struct ThermostatState<'a> {
    pub client: EspMqttClient<'a>,
    pub state_topic: String,
    pub config_topic: String,
    pub command_topic: String,
    pub configuration: ThermostatConfiguration,
    pub state_topic_sub_id: Option<u32>,
    pub command_topic_sub_id: Option<u32>,
    pub pub_id: Option<u32>,
    pub relay: PinDriver<'static, Gpio15, Output>
}

#[derive(Serialize)]
pub struct Device {
    pub identifiers: Vec<String>,
    pub name: String,
    pub sw_version: String,
    pub model: String,
    pub manufacturer: String,
}
    

#[derive(Serialize)]
pub struct ThermostatConfiguration {
    pub payload_off: String,
    pub payload_on: String,
    pub value_template: String,
    pub command_topic: String,
    pub state_topic: String,
    pub name: String,
    pub unique_id: String,
    pub device: Device
}

#[derive(Serialize)]
pub struct ThermostatStateMsg {
    pub state: String
}

#[derive(Debug)]
pub enum ThermostatCommand {
    On,
    Off
}

impl Display for ThermostatCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {Self::On => "ON", Self::Off => "OFF"})
    }
}

impl TryFrom<&str> for ThermostatCommand {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ON" => Ok(Self::On),
            "OFF" => Ok(Self::Off),
            _ => Err("Unrecognized thermostat command")
        }
    }
}
