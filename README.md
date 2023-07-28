# Pixy

[![Build](https://github.com/cryptaliagy/pixy/actions/workflows/build.yaml/badge.svg)](https://github.com/cryptaliagy/pixy/actions/workflows/build.yaml)
[![Crates.io](https://img.shields.io/crates/v/pixy)](https://crates.io/crates/pixy)

Pixy is a web server designed to be used with IoT devices for secure-by-default sensor data relay. It was originally built for use with the Pimoroni Enviro Pico, so you could say it is a Raspberry Pi **Pi**co pro**xy**.

## Overview

Pixy is a Rust-based proxy for relaying sensor messages to multiple targets defined within a simple YAML configuration file. It acts as a single write-only entrypoint for IoT devices, reducing configuration overhead and making it easier to distribute data to many independent systems.

Got suggestions for Pixy? Open an [issue](https://github.com/cryptaliagy/pixy/issues) for a feature request, or start a [discussion](https://github.com/cryptaliagy/pixy/discussions)!

## Motivation

After setting up an Enviro Pico and connecting it to my Home Assistant instance with MQTT, I started thinking of some other uses for it. I've been thinking on how I could connect it up to other automation platforms (like IFTTT), how I might transmit data to citizen science initiatives, or how I might wire up more robust alerting systems, improve fault tolerance, etc., which did not feel possible to do when trying to extend battery life on the Pico as much as possible.

Scouring through some forums and github repositories, I found that other people who use this board also had similar thoughts regarding uploads to multiple targets, as well as increasing the list of available targets. However, multiple upload targets = more time spent uploading = more battery used, so doing this from the Pico itself did not seem like a good idea. I decided that I would write some kind of sensor proxy that could handle the multi-target upload, so the Pico would only ever need to upload to one target.

My goals with this proxy were:

1. The proxy should be _fast_, so the HTTP server listener should return before finishing the re-exporting of the sensor data.
1. The proxy should be _small_, so it could ideally run on a Raspberry Pi Zero.
1. The proxy should be _simple_, so writing a configuration file should be straightforward.
1. The proxy should be _flexible_, so there should be many ways of installing/running it.
1. The proxy should be _scalable_, so it should be able to handle as many or as few boards as you throw at it.

## Installation

Pixy can be installed in a number of different ways depending on your preferences and platform. See the [installation](/docs/Installing.md) docs for more info.

If you are hoping to run Pixy in a Raspberry Pi or a Pi Zero, you should prefer using the Debian/Ubuntu/Raspian instructions that set up the systemd service to run in the background.

At this time, Windows & Mac OS support are only available through the Docker installation.

# Configuring Pixy

After running through the relevant install steps, read the [configuration](/docs/Configuring.md)

## License

This project is primarily licensed under the MIT License. For all terms, please see the [LICENSE](/LICENSE) file.

Some scripts in the [`pkg/debian`](/pkg/debian) directory are based off of code licensed under the MPL 2.0, and as such these are also licensed under the MPL 2.0. Any such scripts under the license will:

1. Include notice that the script is licensed under the MPL 2.0, the full text of which is available in the [LICENSE-MPL](/LICENSE-MPL) file
1. Include a link to the MPL 2.0 FAQ
1. Include a link to the original script used to create the existing script

## Roadmap

For a snapshot of the current plans for Pixy, see the [milestones](https://github.com/cryptaliagy/pixy/milestones) page on GitHub.
