# Configuring Pixy

## Configure your installation

Other than Docker, all Pixy instances are configured based on the flags passed into the Pixy CLI. Use `pixy serve --help` or `man pixy-serve` (if installed with `.deb` package) for information on the available configuration options.

For `.deb` installations, you can change server options by editing the command executed in the unit file stored in `/etc/systemd/system/pixy.service`

For Docker installations, configuration is done through environment variables:

| Variable name    | Default    | Description                                                          |
| ---------------- | ---------- | -------------------------------------------------------------------- |
| PIXY_LOG_LEVEL   | info       | The log level to use. Allowed values are debug/info/warn/error/trace |
| PIXY_PORT        | 9147       | The port that Pixy should listen on                                  |
| PIXY_CONFIG_FILE | /pixy.yaml | The location of the config file                                      |
| PIXY_ENABLE_ECHO | false      | Whether or not to enable the `/echo` route.                          |

The most crucial thing to know before configuring Pixy targets is where the configuration file lives.

- If installed using the `.deb` packages, the Pixy service will read from `/etc/pixy/pixy.yaml`. After changing your configuration, make sure to check it with `pixy validate -c /etc/pixy/pixy.yaml`, and apply your configurations by restarting the service with `sudo systemctl restart pixy.service`
- If installed using Docker, the Pixy service will look for a `/pixy.yaml` file on the container. You will need to mount the file into the container.
- The Pixy CLI will look for a `pixy.yaml` file at the current directory

Example configuration files showing different ways that a `pixy.yaml` file can be configured are available [here](/example-configs/). For a full breakdown of all the types supported by the configuration, see the [types document](/docs/Types.md).

### Setting up the Enviro Pico

Compatible boards:

- Enviro Indoor

During the regular provisioning process, set a custom webhook target and point it at the address of your Pixy server. This will forward the sensor data to all the configured targets in your `pixy.yaml` file in a background task, and immediately return to ensure that your microcontroller wake time is as minimal as possible.

### Running the echo server for debugging

If you would like to enable the echo server that is bundled with Pixy, you can do that in the CLI by using the `--enable-echo` flag (i.e. `pixy serve --enable-echo`), or in the Docker container by setting the `PIXY_ENABLE_ECHO` environment variable to `true`.

The echo server is additionally useful if you are hoping to audit the JSON payload that gets sent from the sensors (by pointing your board's output at the `/echo` route) or if you would like to audit what Pixy is sending to its webhook targets (by adding the `/echo` route to your targets).
