# darwinia-parachain
## Installation
If you just wish to run a darwinia-parachain node without compiling it yourself, you may run the latest binary from our [releases](https://github.com/darwinia-network/darwinia-parachain/releases) page.
### Build from Source
If you'd like to build from source, first install the support software.
```shell
### Debian
sudo apt install --assume-yes git clang curl libssl-dev llvm libudev-dev make protobuf-compiler
### Arch
pacman -Syu --needed --noconfirm curl git clang protobuf
### Fedora
sudo dnf update
sudo dnf install clang curl git openssl-devel protobuf-compiler
### Opensuse
sudo zypper install clang curl git openssl-devel llvm-devel libudev-devel protobuf
```
Once done, Install the Rust toolchain by following Substrate [installation instructions](https://docs.substrate.io/main-docs/install/):
```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default stable
rustup update
rustup update nightly
rustup target add wasm32-unknown-unknown --toolchain nightly
```
Build the client by cloning this repository and running the following commands from the root directory of the repo:
```shell
git checkout <latest tagged release>
cargo build --release
```
## Networks
This repo supports runtimes for darwinia-parachain, crab-parachain, pangolin-parachain.
### Connect to Darwinia Parachain Mainnet
Connect to the global Darwinia Parachain Mainnet network by running:
```shell
./target/release/darwinia-parachain
```
### Connect to the Crab Parachain Canary Network
Connect to the global Crab Parachain canary network by running:
```shell
./target/release/darwinia-parachain --chain crab-parachain
```
### Connect to the Pangolin Parachain Testnet 
Connect to the global Pangolin Parachain testnet by running:
```shell
./target/release/darwinia-parachain --chain pangolin-parachain
```