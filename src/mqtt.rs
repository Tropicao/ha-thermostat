use crate::thermostat::{
    ThermostatCommand, ThermostatConfiguration, ThermostatEvent, ThermostatStateMsg,
};
use esp_idf_svc::mqtt::client::{Details, EspMqttEvent, EventPayload};
use esp_idf_svc::mqtt::client::{EspMqttClient, QoS};
use log::{debug, warn};
use std::str;
use std::sync::mpsc::Sender;

fn process_mqtt_message(data: &[u8], details: Details, sender: &Sender<ThermostatEvent>) {
    match details {
        Details::Complete => {
            debug!("New message: {:?}", data);
            let message = str::from_utf8(data).unwrap();
            let cmd = ThermostatCommand::try_from(message);
            if let Ok(c) = cmd {
                sender.send(ThermostatEvent::CommandReceived(c)).expect("")
            }
        }
        _ => warn!("Unprocessed message event"),
    }
}
pub fn process_mqtt_event(ev: EspMqttEvent, event_tx: &Sender<ThermostatEvent>) {
    match ev.payload() {
        EventPayload::Received {
            id: _,
            topic: _,
            data,
            details,
        } => process_mqtt_message(data, details, event_tx),
        EventPayload::Connected(_) => event_tx.send(ThermostatEvent::MqttConnected).unwrap(),
        EventPayload::Subscribed(id) => {
            event_tx.send(ThermostatEvent::TopicSubscribed(id)).unwrap()
        }
        EventPayload::Published(id) => event_tx
            .send(ThermostatEvent::MessagePublished(id))
            .unwrap(),
        EventPayload::Error(e) => warn!("Received error: {}", e),

        _ => warn!("Unprocessed message {}", ev.payload()),
    }
}

pub fn publish_configuration(
    client: &mut EspMqttClient,
    config_topic: &str,
    configuration: &ThermostatConfiguration,
) -> Result<u32, Box<dyn std::error::Error>> {
    let id = client.publish(
        config_topic,
        QoS::AtLeastOnce,
        false,
        serde_json::to_string(configuration)?.as_bytes(),
    )?;
    Ok(id)
}

pub fn publish_status(
    client: &mut EspMqttClient,
    state_topic: &str,
    cmd: ThermostatCommand,
) -> Result<u32, Box<dyn std::error::Error>> {
    let message = ThermostatStateMsg {
        state: cmd.to_string(),
    };

    let id = client.publish(
        state_topic,
        QoS::AtLeastOnce,
        false,
        serde_json::to_string(&message)?.as_bytes(),
    )?;
    Ok(id)
}
