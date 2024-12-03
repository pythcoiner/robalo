#!/bin/bash

# Exit immediately if any command fails
set -e

SERVICE_NAME="robalo"
USER="pyth"
INSTALL_PATH="/etc"

CURRENT_DIR="$(dirname "$(realpath "$0")")"
BINARY_PATH="/usr/bin/$SERVICE_NAME"
SERVICE_FILE_PATH="$INSTALL_PATH/systemd/system/$SERVICE_NAME.service"
CONF_FILE_PATH="$INSTALL_PATH/$SERVICE_NAME/$SERVICE_NAME.toml"
SERVICE_FILE_TEMPLATE="$CURRENT_DIR/contrib/$SERVICE_NAME.service"
CONF_FILE_TEMPLATE="$CURRENT_DIR/contrib/$SERVICE_NAME.toml"

# Ensure the template toml file exists
if [ ! -f "$CONF_FILE_TEMPLATE" ]; then
  echo "Template configuration file $CONF_FILE_TEMPLATE not found. Aborting."
  exit 1
fi

# Ensure the template service file exists
if [ ! -f "$SERVICE_FILE_TEMPLATE" ]; then
  echo "Template service file $SERVICE_FILE_TEMPLATE not found. Aborting."
  exit 1
fi

# Check if the configuration file exists, if not, copy the template
if [ ! -f "$CONF_FILE_PATH" ]; then
  echo "configuration file not found. Copying from $CONF_FILE_TEMPLATE..."
  if [ ! -d "$INSTALL_PATH/$SERVICE_NAME" ]; then
    sudo mkdir -p "$INSTALL_PATH/$SERVICE_NAME"
  fi
  sudo cp "$CONF_FILE_TEMPLATE" "$CONF_FILE_PATH"
  sudo chmod +rx "$CONF_FILE_PATH"
  echo "configuration file copied to $CONF_FILE_PATH"
fi

# fill placeholder values in the config file
replace_placeholder() {
  local key="$1"
  local placeholder="$2"
  local value
  if grep -q "^$key = .*$placeholder" "$CONF_FILE_PATH"; then
    read -p "Enter value for $key: " value
    if [ -n "$value" ]; then
      sudo sed -i "s|^$key = .*$placeholder|$key = \"$value\"|" "$CONF_FILE_PATH"
    else
      echo "No value entered for $key. Aborting..."
      exit 1
    fi
  fi
}

replace_placeholder "sentry_secret" "<sentry_secret>"
replace_placeholder "mattermost_token" "<mm_token>"
replace_placeholder "mattermost_channel_id" "<mm_channel>"

echo "Building the binary..."
cargo build --release


# remove previous binary
if [ -f "$BINARY_PATH" ]; then
  sudo rm "$BINARY_PATH"
fi

echo "Copying the binary to $BINARY_PATH..."
sudo cp $CURRENT_DIR/target/release/$SERVICE_NAME $BINARY_PATH
sudo chown "$USER:$USER" $BINARY_PATH
sudo chmod +x $BINARY_PATH

echo "Copying the service file to $SERVICE_FILE_PATH..."
sudo cp "$SERVICE_FILE_TEMPLATE" $SERVICE_FILE_PATH
sudo sed -i "s|<user>|$USER|" $SERVICE_FILE_PATH
sudo sed -i "s|<binary_path>|$BINARY_PATH|" $SERVICE_FILE_PATH
sudo sed -i "s|<config_path>|$CONF_FILE_PATH|" $SERVICE_FILE_PATH

echo "Reloading systemd daemon..."
sudo systemctl daemon-reload

echo "Enabling the service to start on boot..."
sudo systemctl enable $SERVICE_NAME

echo "Restarting the service..."
sudo systemctl restart $SERVICE_NAME

sleep 2

# Validate service status
systemctl is-active --quiet $SERVICE_NAME && echo "$SERVICE_NAME is running!" || echo "Failed to start $SERVICE_NAME"

