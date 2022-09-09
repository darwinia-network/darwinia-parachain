# Darwinia Parachain

[![License](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Checks](https://github.com/darwinia-network/darwinia-parachain/actions/workflows/checks.yml/badge.svg?branch=main)](https://github.com/darwinia-network/darwinia-parachain/actions/workflows/checks.yml)
[![Release](https://github.com/darwinia-network/darwinia-parachain/actions/workflows/release.yml/badge.svg?branch=main)](https://github.com/darwinia-network/darwinia-parachain/actions/workflows/release.yml)
[![Quay.io](https://img.shields.io/badge/quay-latest-blue.svg?logo=docker&logoColor=white)](https://quay.io/repository/darwinia-network/darwinia-parachain)
[![GitHub code lines](https://tokei.rs/b1/github/darwinia-network/darwinia-parachain)](https://github.com/darwinia-network/darwinia-parachain)
[![GitHub last commit](https://img.shields.io/github/last-commit/darwinia-network/darwinia-parachain?color=red&style=plastic)](https://github.com/darwinia-network/darwinia-parachain)

This repo contains runitmes for darwinia-parachain, crab-parachain, pangolin-parachain.

## Installation
### Download the Pre-built Binary
[GitHub Release page](https://github.com/darwinia-network/darwinia-parachain/releases)

### Build from Source
Follow the Substrate official [installation instructions](https://docs.substrate.io/main-docs/install) to install the dependencies first.

```sh
git clone https://github.com/darwinia-network/darwinia-parachain.git
cd darwinia-parachain
git checkout <TAG>
cargo build --release
```

## Connect to live networks
- Connect to the global Darwinia Parachain mainnet
    ```sh
    ./target/release/darwinia-parachain
    ```
- Connect to the global Crab Parachain mainnet (canary network of Darwinia Parachain)
    ```sh
    ./target/release/darwinia-parachain --chain crab-parachain
    ```
- Connect to the global Pangolin Parachain testnet
    ```sh
    ./target/release/darwinia-parachain --chain pangolin-parachain
    ```

## Run local testnet with [parachain-launch](https://github.com/open-web3-stack/parachain-launch)
1. Install the package globally
    ```sh
    yarn global add @open-web3/parachain-launch
    ```
2. Generate docker compose files
    ```sh
	git clone https://github.com/darwinia-network/darwinia-parachain.git
	cd darwinia-parachain
    parachain-launch generate --config .maintain/config.yml --yes
    ```
    This command will pull images and generate required docker compose files in a folder called `output` in your current working directory.
3. Start relaychain and parachain
    ```sh
    cd ./output # OR custom output directory

    docker-compose up -d --build
    ```
