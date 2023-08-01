# Types

Below are tables describing types used for context objects and the `pixy.yaml` configuration file.

## Configuration Types

### Config

| Key     | Type                    | Default | Description                                                | Required |
| ------- | ----------------------- | ------- | ---------------------------------------------------------- | -------- |
| targets | list[[Target](#target)] | n/a     | All the targets that Pixy should export the sensor data to | yes      |

### Target

| Key       | Type                | Default | Description                            | Required |
| --------- | ------------------- | ------- | -------------------------------------- | -------- |
| name      | string              | n/a     | The name of the upload target          | yes      |
| enabled   | bool                | true    | Whether or not this target is enabled  | no       |
| webhook\* | [Webhook](#webhook) | n/a     | The configuration for a webhook target | yes      |

> Keys with \* cannot be combined; only one can be specified per target

### Webhook

| Key         | Type                        | Default | Description                                                         | Required |
| ----------- | --------------------------- | ------- | ------------------------------------------------------------------- | -------- |
| url\*       | string                      | n/a     | The URL to post the sensor data to                                  | yes      |
| timeout     | int (1-60)                  | 10      | The number of seconds to wait before timing out a request as failed | no       |
| retries\*\* | int (0-10)                  | 3       | The number of retries when requests fail                            | no       |
| auth        | [WebhookAuth](#webhookauth) | n/a     | Authentication to use with the webhook, if necessary                | no       |

> \* This MUST be an http or https url! You need to include the scheme as a part of the URL
> \*\* Retries use an exponential backoff with jitter to prevent Pixy from spamming downstream targets

#### WebhookAuth

| Key      | Type     | Default | Description                         | Required |
| -------- | -------- | ------- | ----------------------------------- | -------- |
| username | string\* | n/a     | The username to use for Basic Auth. | yes\*\*  |
| password | string\* | n/a     | The password to use for Basic Auth. | yes\*\*  |
| token    | string\* | n/a     | The token to use for Bearer Auth    | yes\*\*  |

> \* These types support using [context objects](/docs/ContextObjects.md).
> \*\* Only one authentication type can be used. If using Basic Auth, set both `username` and `password`. If using Bearer, only set `token`.
