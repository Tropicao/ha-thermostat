# Home Assistant Connected Thermostat

<p align="center">
  <img width="460" height="460" src="https://github.com/Tropicao/ha-thermostat/blob/main/media/cao.png">
</p>
This repository contains instructions and sources to build a simple
thermostat device for a heater which can be driven with dry contact
activation. The thermostat can then be connected to Home Assistant through
MQTT

## Hardware

- An ESP32 board. Those instructions have been successfully implemented on
  an [ESP32 NodeMCU board from
  AZDelivery](https://www.az-delivery.de/fr/products/esp32-developmentboard?_pos=3&_sid=0498583fc&_ss=r)
- A 5V relay module
- An transistor (eg: PN2222) and a resistance (~4.7kOhm) to driver the
  relay module

The ESP32 will be powered through USB and will in turn power the relay
through its 5V pin.

Assemble the hardware:
- connect ESP32 5V pin to 5v input on relay module
- connect ESP32 GND pin to GND input on relay module
- connect ESP32 GPIO 15 to the resistor
- connect resistor other end to transistor base
- connect transistor emitter to ESP32 GND pin
- connect transistor collector to relay module input control pin.

## Software

The repository provides a custom controller written in Rust. It is based on
esp idf providing a std environment to allow writing and running Rust code
on ESP32.

To build and flash the thermostat:
- configure a std environment for esp: follow the [Rust on ESP
  book](https://docs.esp-rs.org/book/installation/index.html) instructions
  to download and install all needed tools
- plug the ESP32 board to your computer with a micro-USB cable
- copy `cfg.toml.example` into a new `cfg.toml` file, and fill needed
  informations regarding your expected setup: wireless credentials, unique
  id to identify your thermostat in Home assistant, IP address of Home
  Assistant MQTT server, etc
- in a console, type `cargo run` to build and flash the thermostat software
  onto the ESP32
- the previous command also opens a serial console onto the ESP32, where
  you can see some logs and ensure that it is properly connecting to your
  Home Assistant.
- if everything has gone fine, you can unplug the thermostat from your
  computer, wire your heater to the relay output pins, and power back the
  thermostat with any smartphone charger.

## Enclosure

The repository also provide a CAD file (for Freecad) for a simple enclosure
that you can 3D print.
