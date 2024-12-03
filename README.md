# Robalo
Robalo is a 'bot' that expose a webhook endpoint (`/alert`) intented to receive 
sentry notifications and forward them to a mattermost server.

# Configuration file
The configuration file must be located at `/etc/robalo/robalo.toml`

Here the configuration file template that can also be found at `./contrib/robalo.toml`:

```toml
ip = "0.0.0.0"
port = 3000
sentry_secret = <sentry_secret>
mattermost_token = <mm_token>
mattermost_channel_id = <mm_channel>
mattermost_base_url = "https://mm.revault.dev"
```

# Requirements

## Rust toolchain 

Rust toolchain should be installed on the machine
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Sentry 

There is 2 main things to configure on sentry side:

 - Create an [`Internal Integration`](https://blog.sentry.io/customize-your-sentry-workflow-a-sample-internal-integration/) 
=> it's basically declaring the webhook endpoint and registering a `secret`, this secret is used
to verify that a notification received on `/alert` endpoint come from sentry.
This secret must be stored in the `sentry_secret` field of `robalo.toml`.

 - Create [`Alerts`](https://docs.sentry.io/product/alerts/create-alerts/) and assign it to
the previously created `Internal Integration`

As of now robalo forward alert of type:
 
 - Issue Created

## Mattermost

There is 3 things to do Mattermost server:

 - Create a (bot) user by SSHing onto the mm server: 

   - `mmctl user create --email <email> --username robalo --password <password> --local`

   - `mmctl user convert robalo --bot --local`

 - Create a `Personnal acces token`for the bot and recort the PAT in the `mattermost_token` field of `robalo.toml`:

   - `mmctl token generate robalo robalo-token --local`

 - Create a channel (and add robalo to the channel) and record its `channel_id` in 
the `mattermost_channel_id` field of `robalo.toml`.

# Installation

simply run `install.sh`, it will:

 - Install the configuration file if it's the first install

 - Ask user for  sentry token & mattermost token/channel_id

 - Build binary

 - Copy binary to /usr/bin/

 - Write the systemd service file

 - Configure and start the systemd service


 `sudo` is needed for:

  - copy the binary in `/usr/bin/`

  - create the systemd service file at `/etc/systemd/system/robalo.service`

  - create the configuration file at `/etc/robalo/robalo.toml`

but the script should **NOT** be run as root/sudo itself, sudo is only used for previous commands by the script itself.
