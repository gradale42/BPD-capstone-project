#!/bin/bash

# Setup  pre-req
if command -v cargo > /dev/null 2>&1; then
    echo "Cargo is already installed. Current version: $(cargo --version)"
else
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  source $HOME/.cargo/env
fi

set -e  # Exit immediately if any command fails

# Start docker
export COMPOSE_PROFILES=regtest,signet
docker compose -f ./docker/docker-compose.yml up -d
echo " Docker started."

sleep 5

./scripts/smart_fund.sh
./scripts/init_db.sh

cargo run


