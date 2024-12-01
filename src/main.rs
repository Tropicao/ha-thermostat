use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{gpio::PinDriver, prelude::Peripherals},
    mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS},
};
use log::{debug, info};
use std::{str, sync::mpsc};

mod mqtt;
mod thermostat;
mod wifi;

use thermostat::{
    Device, ThermostatCommand, ThermostatConfiguration, ThermostatEvent, ThermostatState,
};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("SSID_DEFAULT")]
    wifi_ssid: &'static str,
    #[default("PSK_DEFAULT")]
    wifi_psk: &'static str,
    #[default("000000000000000000000000000000000000")]
    uuid: &'static str,
    #[default("127.0.0.1")]
    mqtt_host: &'static str,
}

fn process_thermostat_event(state: &mut ThermostatState, event: ThermostatEvent) {
    debug!("New message {:?}", event);
    match event {
        ThermostatEvent::MqttConnected => {
            info!("Thermostat connected, subscribing to command topic...");
            state.command_topic_sub_id = state
                .client
                .subscribe(&state.command_topic, QoS::AtLeastOnce)
                .ok();
        }
        ThermostatEvent::TopicSubscribed(id) => {
            if let Some(state_id) = state.state_topic_sub_id {
                if state_id == id {
                    info!("State topic subscribed, publishing thermostat configuration...");
                    state.state_topic_sub_id = None;
                    state.pub_id = mqtt::publish_configuration(
                        &mut state.client,
                        &state.config_topic,
                        &state.configuration,
                    )
                    .ok();
                }
            } else if let Some(command_id) = state.command_topic_sub_id {
                if command_id == id {
                    info!("Command topic subscribed, subscribing to state topic...");
                    state.command_topic_sub_id = None;
                    state.state_topic_sub_id = state
                        .client
                        .subscribe(&state.state_topic, QoS::AtLeastOnce)
                        .ok();
                }
            }
        }
        ThermostatEvent::MessagePublished(id) => {
            if let Some(pub_id) = state.pub_id {
                if pub_id == id {
                    info!("Message {:?} published", id);
                    state.pub_id = None;
                }
            }
        }
        ThermostatEvent::CommandReceived(cmd) => {
            info!("New thermostat command {:?}, publishing new state...", cmd);
            match cmd {
                ThermostatCommand::On => state.relay.set_high(),
                ThermostatCommand::Off => state.relay.set_low(),
            }
            .unwrap();
            state.pub_id = mqtt::publish_status(&mut state.client, &state.state_topic, cmd).ok();
        }
    }
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let (event_tx, event_rx) = mpsc::channel();
    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;
    let app_config = CONFIG;

    info!("Initialising device {:?}", app_config.uuid);

    let _wifi = wifi::configure_wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
    )?;
    let broker_url = format!("mqtt://{}", app_config.mqtt_host);
    let mqtt_config = MqttClientConfiguration::<'_> {
        skip_cert_common_name_check: true,
        ..Default::default()
    };
    let state_topic = "custom_devices/heater";
    let command_topic = "custom_devices/heater/set";
    let mut state = ThermostatState {
        state_topic_sub_id: None,
        command_topic_sub_id: None,
        pub_id: None,
        client: EspMqttClient::new_cb(&broker_url, &mqtt_config, move |ev| {
            mqtt::process_mqtt_event(ev, &event_tx)
        })?,
        command_topic: String::from(command_topic),
        state_topic: String::from(state_topic),
        config_topic: format!("homeassistant/switch/{}/switch/config", app_config.uuid),
        relay: PinDriver::output(peripherals.pins.gpio15)?,
        configuration: ThermostatConfiguration {
            payload_off: String::from("OFF"),
            payload_on: String::from("ON"),
            value_template: String::from("{{ value_json.state }}"),
            state_topic: String::from(state_topic),
            command_topic: String::from(command_topic),
            name: format!("{}_switch", app_config.uuid),
            unique_id: String::from(app_config.uuid),
            device: Device {
                identifiers: Vec::from([format!("custom_device_{}", app_config.uuid)]),
                name: String::from(app_config.uuid),
                sw_version: String::from("1.0"),
                model: String::from("Thermostat"),
                manufacturer: String::from("Tropicao"),
            },
        },
    };

    loop {
        match event_rx.recv() {
            Ok(msg) => process_thermostat_event(&mut state, msg),
            Err(e) => panic!("{:?}", e),
        }
    }
}
