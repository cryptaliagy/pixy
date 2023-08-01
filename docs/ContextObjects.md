# Context Objects

## Using Context Objects

Sometimes it is important to use information that you don't want to directly hardcode in your `pixy.yaml` file, or use data that depends on the information sent to Pixy by the sensors. One example of this is passwords- storing passwords in the configuration file itself is considered to be an unsafe practice, especially since configuration files tend to be stored in things like Github.

Examples of configurations using context objects can be found [here](/example-configs/webhook.yaml), or a minimal example is shown below for a webhook target.

```yaml
targets:
  - name: "A basic webhook target"
    webhook:
      url: "http://localhost:9147/echo"
      auth:
        token: "{{ env.TOKEN }}"
```

## Supported Context Objects

Below are all the context objects currently supported. These can be used anywhere that context objects are supported.

| Name      | Description                                                                            |
| --------- | -------------------------------------------------------------------------------------- |
| `env`     | Contains all environment variables that are prefixed with `PIXY_`, removing the prefix |
| `message` | A [SensorMessage](#sensormessage) containing information sent in the messsage to Pixy  |

## Context Object Types

### SensorMessage

| Key       | Type                                | Description                                            | Nullable |
| --------- | ----------------------------------- | ------------------------------------------------------ | -------- |
| readings  | [Reading](#reading)                 | The values recorded by the sensors                     | no       |
| timestamp | string                              | The time the reading was taken                         | no       |
| metadata  | [MessageMetadata](#messagemetadata) | Information about the device that produced the message | no       |

### Reading

| Key               | Type | Description                          | Nullable |
| ----------------- | ---- | ------------------------------------ | -------- |
| temperature       | f32  | A temperature reading, in Celsius    | no       |
| pressure          | f32  | The pressure reading, in hPa         | no       |
| humidity          | f32  | The humidity, in relative percentage | no       |
| color_temperature | u64  | The color temperature, in Kelvin     | yes      |
| gas_resistance    | u64  | The gas resistance, in Ohms          | yes      |
| aqi               | f32  | The Indoor Air Quality score         | yes      |
| luminance         | u64  | The luminance, in lux                | yes      |

### MessageMetadata

| Key      | Type   | Description                                        | Nullable |
| -------- | ------ | -------------------------------------------------- | -------- |
| nickname | string | The nickname of the board that sent this message   | no       |
| model    | string | The model name of the board that sent this message | no       |
| uid      | string | A unique ID for the board that sent this message   | no       |
