## Installation - Debian/Ubuntu/Raspian (Recommended)

> `*_armel.deb` architecture packages are considered experimental at this time since I do not own devices that I can use to properly test on that architecture. If you are able to help with testing Pixy on these devices, I encourage you to reach out!

1. Head to the [releases page](https://github.com/cryptaliagy/pixy/releases/latest) and choose the version you would like. Make sure to note the architecture of your system
1. Run the command `wget https://github.com/cryptaliagy/pixy/releases/download/<VERSION>/pixy_<VERSION>_<ARCH>.deb`, filling in the right version and architecture
1. Run `sudo dpkg -i pixy_<VERSION>_<ARCH>.deb`, filling in the right version and architecture
1. Confirm that the installation was successful by running `pixy --version`
1. (Optional) Enable the Pixy service with `sudo systemctl enable pixy`
1. (Optional) Start the Pixy service with `sudo systemctl start pixy`
1. (Optional) Confirm that the Pixy service is running with `sudo systemctl status pixy`
1. (Optional) Add yourself to the `pixy` group to get permissions to modify the file with `sudo usermod -aG pixy $USER`, then log out and log back in

Installing the `.deb` package will include the `systemd` unit file to run the Pixy service, as well as `man` files for the `pixy` command and its subcommands. This will also set up the Pixy user on the system, the config directories, and the data directory.

For more information, use `man pixy` after install.

## Installation - Direct Download

> `*.armv7` binaries are considered experimental at this time since I do not own devices that I can use to properly test on that architecture. If you are able to help with testing Pixy on these devices, I encourage you to reach out!

1. Head to the [releases page](https://github.com/cryptaliagy/pixy/releases/latest) and choose the version you would like. Make sure to note the architecture of your system
1. Run the command `wget https://github.com/cryptaliagy/pixy/releases/download/<VERSION>/pixy.<ARCH>`, filling in the right version and architecture
1. Run `sudo mv ./pixy.<ARCH> /usr/bin/pixy`
1. Run `sudo chmod 755 /usr/bin/pixy` to set the permissions as executable
1. Run `pixy --version` to confirm that the executable is installed and discoverable

## Installation - Cargo (binstall)

> Supported for pixy >= v0.2.0

1. Run the command `cargo binstall pixy`
1. Run `pixy --version` to confirm that the executable is installed and discoverable

## Installation - Cargo (install)

Support for this installation method is only provided for the following targets:

- `x86_64-unknown-musl`
- `aarch64-unknown-musl`

> Make sure the `musl-tools` package is installed on the system!

1. Run `cargo install pixy --target <target>`
1. Confirm that the installation was successful by running `pixy --version`

## Installation - Docker (Advanced)

> Only `amd64` platforms are currently supported for Docker. See [this issue](https://github.com/cryptaliagy/pixy/issues/9)

1. Write a configuration file `pixy.yaml` and save it to your current directory. See the [example configs](/example-configs/) directory to get started, or the section on [writing a `pixy.yaml` file](#writing-a-pixyyaml-file)
1. Pull the Docker container with `docker pull ghcr.io/cryptaliagy/pixy:latest`
1. Run the Docker container with `docker run -v ./pixy.yaml:/pixy.yaml -p 8080:8080 ghcr.io/cryptaliagy/pixy:latest`
