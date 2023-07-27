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

### Writing a `pixy.yaml` File

The most crucial thing to know before configuring Pixy targets is where the configuration file lives.

- If installed using the `.deb` packages, the Pixy service will read from `/etc/pixy/pixy.yaml`. After changing your configuration, make sure to check it with `pixy validate -c /etc/pixy/pixy.yaml`, and apply your configurations by restarting the service with `sudo systemctl restart pixy.service`
- If installed using Docker, the Pixy service will look for a `/pixy.yaml` file on the container. You need to mount the file into the container.
- The Pixy CLI will look for a `pixy.yaml` file at the current directory

Below are tables describing the various types and fields allowed in the config file.

Config file:

| Key     | Type         | Default | Description                                                | Required |
| ------- | ------------ | ------- | ---------------------------------------------------------- | -------- |
| targets | list[Target] | n/a     | All the targets that Pixy should export the sensor data to | yes      |

Target type:

| Key       | Type    | Default | Description                            | Required |
| --------- | ------- | ------- | -------------------------------------- | -------- |
| name      | string  | n/a     | The name of the upload target          | yes      |
| enabled   | bool    | true    | Whether or not this target is enabled  | no       |
| webhook\* | Webhook | n/a     | The configuration for a webhook target | yes      |

> Keys with \* cannot be combined; only one can be specified per target

Webhook type:

| Key           | Type       | Default | Description                                                         | Required |
| ------------- | ---------- | ------- | ------------------------------------------------------------------- | -------- |
| url\*\*       | url        | n/a     | The URL to post the sensor data to                                  | yes      |
| timeout       | int (1-60) | 10      | The number of seconds to wait before timing out a request as failed | no       |
| retries\*\*\* | int (0-10) | 3       | The number of retries when requests fail                            | no       |

> \*\* This MUST be an http or https url! You need to include the scheme as a part of the URL
> \*\*\* Retries use an exponential backoff with jitter to prevent Pixy from spamming downstream targets

### Setting up the Enviro Pico

Compatible boards:

- Enviro Indoor

During the regular provisioning process, set a custom webhook target and point it at the address of your Pixy server. This will forward the sensor data to all the configured targets in your `pixy.yaml` file in a background task, and immediately return to ensure that your microcontroller wake time is as minimal as possible.

### Running the echo server for debugging

If you would like to enable the echo server that is bundled with Pixy, you can do that in the CLI by using the `--enable-echo` flag (i.e. `pixy serve --enable-echo`), or in the Docker container by setting the `PIXY_ENABLE_ECHO` environment variable to `true`.

The echo server is additionally useful if you are hoping to audit the JSON payload that gets sent from the sensors (by pointing your board's output at the `/echo` route) or if you would like to audit what Pixy is sending to its webhook targets (by adding the `/echo` route to your targets).
