#!/bin/bash
# this script must be run as root

# Exit immediately if any command fails
set -e

SERVICE_NAME="robalo"
BINARY_PATH="/usr/bin/$SERVICE_NAME"
SERVICE_FILE_PATH="/etc/systemd/system/$SERVICE_NAME.service"
CURRENT_DIR="$(dirname "$(realpath "$0")")"

echo "Building the binary..."
cargo build --release

echo "Copying the binary to $BINARY_PATH..."
cp target/release/$SERVICE_NAME $BINARY_PATH
chown www-data:www-data $BINARY_PATH
chmod +x $BINARY_PATH

echo "Copying the service file to $SERVICE_FILE_PATH..."
cp "$CURRENT_DIR/$SERVICE_NAME.service" $SERVICE_FILE_PATH

echo "Reloading systemd daemon..."
systemctl daemon-reload

echo "Restarting the service..."
systemctl restart $SERVICE_NAME

echo "Enabling the service to start on boot..."
systemctl enable $SERVICE_NAME

echo "$SERVICE_NAME is running!"
