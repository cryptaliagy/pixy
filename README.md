# Pixy

Pixy is a web server designed to be used with IoT devices for secure-by-default sensor data relay. It was originally built for use with the Pimoroni Enviro Pico, so you could say it is a Raspberry Pi **Pi**co pro**xy**.

Note, installing Pixy on Windows is only officially supported through Docker

## Motivation

## Installation - Debian/Ubuntu/Raspian

> `armel.deb` architecture packages are considered experimental at this time since I do not own devices that I can use to properly test on that architecture. If you are able to help with testing Pixy on these devices, I encourage you to reach out!

1. Head to the [releases page](https://github.com/cryptaliagy/pixy/releases/latest) and choose the version you would like. Make sure to note the architecture of your system
1. Run the command `wget https://github.com/cryptaliagy/pixy/releases/download/<VERSION>/pixy_<VERSION>_<ARCH>.deb`
1. Run `sudo dpkg -i pixy_<VERSION>_<ARCH>.deb`
1. Confirm that the installation was successful by running `pixy --version`
1. Confirm that the pixy service is running by using `sudo service status pixy`

Skip to [overview](#overview)

## Installation - Docker

1. Write a configuration file `pixy.yaml` and save it to your current directory. See the [example configs](/example-configs/) directory to get started, or the section on [writing a `pixy.yaml` file](#writing-a-pixyyaml-file)
1. Pull the Docker container with `docker pull ghcr.io/cryptaliagy/pixy:latest`
1. Run the Docker container with `docker run -v ./pixy.yaml:/pixy.yaml -p 8080:8080 ghcr.io/cryptaliagy/pixy:latest`

Skip to [overview](#overview)

## Installation - Cargo

Support for this installation method is only provided for the following targets:

- `x86_64-unknown-musl`
- `aarch64-unknown-musl`

1. Run `cargo install pixy --target <target>`
1. Confirm that the installation was successful by running `pixy --version`

> NOTE: This will not install the Pixy `systemd` service!

Skip to [overview](#overview)

## Overview

### Writing a `pixy.yaml` file

### Running the echo server for debugging

If you would like to enable the echo server that is bundled with Pixy, you can do that in the CLI by using the `--enable-echo` flag (i.e. `pixy server --enable-echo`), or in the Docker container by setting the `PIXY_ENABLE_ECHO` environment variable to `true`.

The echo server is additionally useful if you are hoping to audit the JSON payload that gets sent from the sensors (by pointing your board's output at the `/echo` route) or if you would like to audit what Pixy is sending to its webhook targets (by adding the `/echo` route to your targets).

### Setting up the Enviro Pico

Compatible boards:

- Enviro Indoor

During the regular provisioning process, set a custom webhook target and point it at the address of your Pixy server. This will forward the sensor data to all the configured targets in your `pixy.yaml` file in a background task, and immediately return to ensure that your microcontroller wake time is as minimal as possible.

## License

This project is licensed under the MIT License. For all terms, please see the [LICENSE](/LICENSE) file.

## Contributing
