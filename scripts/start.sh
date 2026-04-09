#!/bin/bash

# Setup  pre-req
if command -v cargo > /dev/null 2>&1; then
    echo "Cargo is already installed. Current version: $(cargo --version)"
else
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  source $HOME/.cargo/env
fi

set -e  # Exit immediately if any command fails

mkdir -p ./docker/datadir
if [ ! -d "./docker/datadir/node0" ]; then
    cp -r ./datadir/node0 ./docker/datadir/
fi

# Start docker
docker compose -f ./docker/docker-compose.yml up -d
echo " Docker started."

sleep 5

./scripts/smart_fund.sh
./scripts/init_db.sh

cargo run


