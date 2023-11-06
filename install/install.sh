#!/bin/bash

UNAME_OUTPUT="$(uname -s)"
case "${UNAME_OUTPUT}" in
    Darwin*)    ARCHIVE_FILENAME=space-x86_64-apple-darwin.tar.gz;;
    *)          ARCHIVE_FILENAME=space-x86_64-unknown-linux-musl.tar.gz;;
esac

ARCHIVE_FILE_URL=https://github.com/emilevr/space/releases/latest/download/$ARCHIVE_FILENAME
ARCHIVE_DIR=$HOME
ARCHIVE_FILE_PATH=$ARCHIVE_DIR/$ARCHIVE_FILENAME
BINARY_FILE_PATH=$ARCHIVE_DIR/space
INSTALL_DIR=/usr/local/bin
INSTALL_FILE_PATH=$INSTALL_DIR/space

echo "Installing space CLI to $INSTALL_DIR"

echo "⬇⬇ Downloading the latest binary from $ARCHIVE_FILE_URL ..."
(curl -L -o $ARCHIVE_FILE_PATH $ARCHIVE_FILE_URL) || \
    { echo "❌ Download failed!"; exit 1; }

echo "👷 Extacting downloaded file $ARCHIVE_FILE_PATH"
(tar -xzvf $ARCHIVE_FILE_PATH -C $ARCHIVE_DIR/) || \
    { echo "❌ Could not extract archive!"; exit 2; }

echo "👷 Making the binary executable"
(chmod +x $BINARY_FILE_PATH) || \
    { echo "❌ Could not make space executable!"; exit 3; }

echo "👷 Moving $BINARY_FILE_PATH to $INSTALL_DIR"
(sudo mv $BINARY_FILE_PATH $INSTALL_FILE_PATH) || \
    { echo "❌ Failed to move $BINARY_FILE_PATH to $INSTALL_DIR!"; exit 4; }

echo "👷 Cleaning up..."
(rm -f $ARCHIVE_FILE_PATH) || \
    { echo "❌ Cleanup failed!"; exit 5; }

echo "✔️ Installation completed. Run 'space --help' to see a list of available options."
