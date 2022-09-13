// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

// --- std ---
use std::{env, net::SocketAddr, path::PathBuf};
// --- crates.io ---
use codec::Encode;
use log::info;
// --- paritytech ---
use cumulus_client_cli::generate_genesis_block;
use cumulus_primitives_core::ParaId;
use frame_benchmarking_cli::{BenchmarkCmd, SUBSTRATE_REFERENCE_HARDWARE};
use sc_cli::{
	ChainSpec, CliConfiguration, DefaultConfigurationValues, ImportParams, KeystoreParams,
	NetworkParams, Result, RuntimeVersion, SharedParams, SubstrateCli,
};
use sc_service::config::{BasePath, PrometheusConfig};
use sp_core::{crypto::Ss58AddressFormatRegistry, hexdisplay::HexDisplay};
use sp_runtime::traits::{AccountIdConversion, Block as BlockT};
// --- darwinia-network ---
use crate::{
	chain_spec::*,
	cli::*,
	service::{self, *},
};
use dc_primitives::{AccountId, OpaqueBlock as Block};

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Darwinia Parachain".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/darwinia-network/darwinia-parachain/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2018
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn ChainSpec>, String> {
		load_spec(id)
	}

	fn native_runtime_version(spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		if spec.is_crab_parachain() {
			&crab_parachain_runtime::VERSION
		} else if spec.is_pangolin_parachain() {
			&pangolin_parachain_runtime::VERSION
		} else {
			&darwinia_parachain_runtime::VERSION
		}
	}
}

impl SubstrateCli for RelayChainCli {
	fn impl_name() -> String {
		"Darwinia Parachain".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/darwinia-network/darwinia-parachain/issues/new".into()
	}

	fn copyright_start_year() -> i32 {
		2018
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		polkadot_cli::Cli::from_iter([RelayChainCli::executable_name()].iter()).load_spec(id)
	}

	fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		polkadot_cli::Cli::native_runtime_version(chain_spec)
	}
}

fn load_spec(id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
	let id = if id.is_empty() {
		let n = get_exec_name().unwrap_or_default();
		["darwinia-parachain", "crab-parachain", "pangolin-parachain"]
			.iter()
			.cloned()
			.find(|&chain| n.starts_with(chain))
			.unwrap_or("darwinia-parachain")
	} else {
		id
	};

	Ok(match id.to_lowercase().as_ref() {
		"darwinia-parachain" => Box::new(darwinia_parachain_chain_spec::config()?),
		"darwinia-parachain-genesis" => Box::new(darwinia_parachain_chain_spec::genesis_config()),
		"darwinia-parachain-dev" => Box::new(darwinia_parachain_chain_spec::development_config()),
		"crab-parachain" => Box::new(crab_parachain_chain_spec::config()?),
		"crab-parachain-genesis" => Box::new(crab_parachain_chain_spec::genesis_config()),
		"crab-parachain-dev" => Box::new(crab_parachain_chain_spec::development_config()),
		"pangolin-parachain" => Box::new(pangolin_parachain_chain_spec::config()?),
		"pangolin-parachain-genesis" => Box::new(pangolin_parachain_chain_spec::genesis_config()),
		"pangolin-parachain-dev" => Box::new(pangolin_parachain_chain_spec::development_config()),
		_ => {
			let path = PathBuf::from(id);
			let chain_spec = Box::new(DarwiniaParachainChainSpec::from_json_file(path.clone())?)
				as Box<dyn ChainSpec>;

			if chain_spec.is_crab_parachain() {
				Box::new(CrabParachainChainSpec::from_json_file(path)?)
			} else if chain_spec.is_pangolin_parachain() {
				Box::new(PangolinParachainChainSpec::from_json_file(path)?)
			} else {
				chain_spec
			}
		},
	})
}

fn get_exec_name() -> Option<String> {
	env::current_exe()
		.ok()
		.and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
		.and_then(|s| s.into_string().ok())
}

fn set_default_ss58_version(chain_spec: &dyn IdentifyVariant) {
	let ss58_version = if chain_spec.is_crab_parachain() || chain_spec.is_pangolin_parachain() {
		Ss58AddressFormatRegistry::SubstrateAccount
	} else {
		Ss58AddressFormatRegistry::DarwiniaAccount
	}
	.into();

	sp_core::crypto::set_default_ss58_version(ss58_version);
}

/// Parse command line arguments into service configuration.
pub fn run() -> Result<()> {
	/// Creates partial components for the runtimes that are supported by the benchmarks.
	macro_rules! construct_benchmark_partials {
		($config:expr, |$partials:ident| $code:expr) => {
			if $config.chain_spec.is_crab_parachain() {
				let $partials = new_partial::<CrabParachainRuntimeApi, _>(
					&$config,
					service::build_import_queue,
				)?;
				$code
			} else if $config.chain_spec.is_pangolin_parachain() {
				let $partials = new_partial::<PangolinParachainRuntimeApi, _>(
					&$config,
					service::build_import_queue,
				)?;
				$code
			} else {
				let $partials = new_partial::<DarwiniaParachainRuntimeApi, _>(
					&$config,
					service::build_import_queue,
				)?;
				$code
			}
		};
	}

	macro_rules! construct_async_run {
		(|$components:ident, $cli:ident, $cmd:ident, $config:ident| $( $code:tt )* ) => {{
			let runner = $cli.create_runner($cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if chain_spec.is_crab_parachain() {
				runner.async_run(|$config| {
					let $components = new_partial::<CrabParachainRuntimeApi, _>(
						&$config,
						service::build_import_queue,
					)?;
					let task_manager = $components.task_manager;
					{ $( $code )* }.map(|v| (v, task_manager))
				})
			} else if chain_spec.is_pangolin_parachain() {
				runner.async_run(|$config| {
					let $components = new_partial::<PangolinParachainRuntimeApi, _>(
						&$config,
						service::build_import_queue,
					)?;
					let task_manager = $components.task_manager;
					{ $( $code )* }.map(|v| (v, task_manager))
				})
			} else {
				runner.async_run(|$config| {
					let $components = new_partial::<DarwiniaParachainRuntimeApi, _>(
						&$config,
						service::build_import_queue,
					)?;
					let task_manager = $components.task_manager;
					{ $( $code )* }.map(|v| (v, task_manager))
				})
			}
		}}
	}

	let cli = Cli::from_args();

	match &cli.subcommand {
		None => cli.create_runner(&cli.run.normalize())?.run_node_until_exit(|config| async move {
			let chain_spec = &config.chain_spec;
			let collator_options = cli.run.collator_options();

			let hwbench = if !cli.no_hardware_benchmarks {
				config.database.path().map(|database_path| {
					let _ = std::fs::create_dir_all(&database_path);
					sc_sysinfo::gather_hwbench(Some(database_path))
				})
			} else {
				None
			};

			set_default_ss58_version(chain_spec);

			let para_id = Extensions::try_get(&*config.chain_spec)
				.map(|e| e.para_id)
				.ok_or("Could not find parachain ID in chain-spec.")?;
			let polkadot_cli = RelayChainCli::new(
				&config,
				[RelayChainCli::executable_name()].iter().chain(cli.relay_chain_args.iter()),
			);
			let id = ParaId::from(para_id);
			let parachain_account = AccountIdConversion::<AccountId>::into_account_truncating(&id);
			let state_version = Cli::native_runtime_version(&config.chain_spec).state_version();
			let block: Block = generate_genesis_block(&*config.chain_spec, state_version)
				.map_err(|e| format!("{:?}", e))?;
			let genesis_state = format!("0x{:?}", HexDisplay::from(&block.header().encode()));
			let tokio_handle = config.tokio_handle.clone();
			let polkadot_config =
				SubstrateCli::create_configuration(&polkadot_cli, &polkadot_cli, tokio_handle)
					.map_err(|err| format!("Relay chain argument error: {}", err))?;

			info!("Parachain id: {:?}", id);
			info!("Parachain Account: {}", parachain_account);
			info!("Parachain genesis state: {}", genesis_state);
			info!("Is collating: {}", if config.role.is_authority() { "yes" } else { "no" });

			if chain_spec.is_crab_parachain() {
				service::start_node::<CrabParachainRuntimeApi>(
					config,
					polkadot_config,
					collator_options,
					id,
					hwbench,
				)
				.await
				.map(|r| r.0)
				.map_err(Into::into)
			} else if chain_spec.is_pangolin_parachain() {
				service::start_node::<PangolinParachainRuntimeApi>(
					config,
					polkadot_config,
					collator_options,
					id,
					hwbench,
				)
				.await
				.map(|r| r.0)
				.map_err(Into::into)
			} else {
				service::start_node::<DarwiniaParachainRuntimeApi>(
					config,
					polkadot_config,
					collator_options,
					id,
					hwbench,
				)
				.await
				.map(|r| r.0)
				.map_err(Into::into)
			}
		}),
		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},
		Some(Subcommand::CheckBlock(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		},
		Some(Subcommand::ExportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, config.database))
			})
		},
		Some(Subcommand::ExportState(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, config.chain_spec))
			})
		},
		Some(Subcommand::ImportBlocks(cmd)) => {
			construct_async_run!(|components, cli, cmd, config| {
				Ok(cmd.run(components.client, components.import_queue))
			})
		},
		Some(Subcommand::Revert(cmd)) => construct_async_run!(|components, cli, cmd, config| {
			Ok(cmd.run(components.client, components.backend, None))
		}),
		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				let polkadot_cli = RelayChainCli::new(
					&config,
					[RelayChainCli::executable_name()].iter().chain(cli.relay_chain_args.iter()),
				);

				let polkadot_config = SubstrateCli::create_configuration(
					&polkadot_cli,
					&polkadot_cli,
					config.tokio_handle.clone(),
				)
				.map_err(|err| format!("Relay chain argument error: {}", err))?;

				cmd.run(config, polkadot_config)
			})
		},
		Some(Subcommand::ExportGenesisState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|_config| {
				let spec = cli.load_spec(&cmd.shared_params.chain.clone().unwrap_or_default())?;
				let state_version = Cli::native_runtime_version(&spec).state_version();
				cmd.run::<Block>(&*spec, state_version)
			})
		},
		Some(Subcommand::ExportGenesisWasm(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|_config| {
				let spec = cli.load_spec(&cmd.shared_params.chain.clone().unwrap_or_default())?;
				cmd.run(&*spec)
			})
		},
		Some(Subcommand::Key(cmd)) => Ok(cmd.run(&cli)?),
		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			// Switch on the concrete benchmark sub-command-
			match cmd {
				BenchmarkCmd::Pallet(cmd) =>
					if cfg!(feature = "runtime-benchmarks") {
						runner.sync_run(|config| {
							if config.chain_spec.is_crab_parachain() {
								cmd.run::<Block, CrabParachainRuntimeExecutor>(config)
							} else if config.chain_spec.is_pangolin_parachain() {
								cmd.run::<Block, PangolinParachainRuntimeExecutor>(config)
							} else {
								cmd.run::<Block, DarwiniaParachainRuntimeExecutor>(config)
							}
						})
					} else {
						Err("Benchmarking wasn't enabled when building the node. \
				You can enable it with `--features runtime-benchmarks`."
							.into())
					},
				BenchmarkCmd::Block(cmd) => runner.sync_run(|config| {
					construct_benchmark_partials!(config, |partials| cmd.run(partials.client))
				}),
				BenchmarkCmd::Storage(cmd) => runner.sync_run(|config| {
					construct_benchmark_partials!(config, |partials| {
						let db = partials.backend.expose_db();
						let storage = partials.backend.expose_storage();

						cmd.run(config, partials.client.clone(), db, storage)
					})
				}),
				BenchmarkCmd::Overhead(_) => Err("Unsupported benchmarking command".into()),
				BenchmarkCmd::Machine(cmd) =>
					runner.sync_run(|config| cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())),
			}
		},
		#[cfg(feature = "try-runtime")]
		Some(Subcommand::TryRuntime(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			let chain_spec = &runner.config().chain_spec;

			set_default_ss58_version(chain_spec);

			if chain_spec.is_crab_parachain() {
				runner.async_run(|config| {
					// we don't need any of the components of new_partial, just a runtime, or a task
					// manager to do `async_run`.
					let registry = config.prometheus_config.as_ref().map(|cfg| &cfg.registry);
					let task_manager =
						sc_service::TaskManager::new(config.tokio_handle.clone(), registry)
							.map_err(|e| sc_cli::Error::from(sc_service::Error::Prometheus(e)))?;

					Ok((cmd.run::<Block>(config), task_manager))
				})
			} else {
				panic!("Try runtime not support chain: {}", chain_spec.id());
			}
		},
	}
}

impl DefaultConfigurationValues for RelayChainCli {
	fn p2p_listen_port() -> u16 {
		30334
	}

	fn rpc_ws_listen_port() -> u16 {
		9945
	}

	fn rpc_http_listen_port() -> u16 {
		9934
	}

	fn prometheus_listen_port() -> u16 {
		9616
	}
}

impl CliConfiguration<Self> for RelayChainCli {
	fn shared_params(&self) -> &SharedParams {
		self.base.base.shared_params()
	}

	fn import_params(&self) -> Option<&ImportParams> {
		self.base.base.import_params()
	}

	fn network_params(&self) -> Option<&NetworkParams> {
		self.base.base.network_params()
	}

	fn keystore_params(&self) -> Option<&KeystoreParams> {
		self.base.base.keystore_params()
	}

	fn base_path(&self) -> Result<Option<BasePath>> {
		Ok(self.shared_params().base_path().or_else(|| self.base_path.clone().map(Into::into)))
	}

	fn rpc_http(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_http(default_listen_port)
	}

	fn rpc_ipc(&self) -> Result<Option<String>> {
		self.base.base.rpc_ipc()
	}

	fn rpc_ws(&self, default_listen_port: u16) -> Result<Option<SocketAddr>> {
		self.base.base.rpc_ws(default_listen_port)
	}

	fn prometheus_config(
		&self,
		default_listen_port: u16,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<PrometheusConfig>> {
		self.base.base.prometheus_config(default_listen_port, chain_spec)
	}

	fn init<F>(
		&self,
		_support_url: &String,
		_impl_version: &String,
		_logger_hook: F,
		_config: &sc_service::Configuration,
	) -> Result<()>
	where
		F: FnOnce(&mut sc_cli::LoggerBuilder, &sc_service::Configuration),
	{
		unreachable!("PolkadotCli is never initialized; qed");
	}

	fn chain_id(&self, is_dev: bool) -> Result<String> {
		let chain_id = self.base.base.chain_id(is_dev)?;

		Ok(if chain_id.is_empty() { self.chain_id.clone().unwrap_or_default() } else { chain_id })
	}

	fn role(&self, is_dev: bool) -> Result<sc_service::Role> {
		self.base.base.role(is_dev)
	}

	fn transaction_pool(&self) -> Result<sc_service::config::TransactionPoolOptions> {
		self.base.base.transaction_pool()
	}

	fn state_cache_child_ratio(&self) -> Result<Option<usize>> {
		self.base.base.state_cache_child_ratio()
	}

	fn rpc_methods(&self) -> Result<sc_service::config::RpcMethods> {
		self.base.base.rpc_methods()
	}

	fn rpc_ws_max_connections(&self) -> Result<Option<usize>> {
		self.base.base.rpc_ws_max_connections()
	}

	fn rpc_cors(&self, is_dev: bool) -> Result<Option<Vec<String>>> {
		self.base.base.rpc_cors(is_dev)
	}

	fn default_heap_pages(&self) -> Result<Option<u64>> {
		self.base.base.default_heap_pages()
	}

	fn force_authoring(&self) -> Result<bool> {
		self.base.base.force_authoring()
	}

	fn disable_grandpa(&self) -> Result<bool> {
		self.base.base.disable_grandpa()
	}

	fn max_runtime_instances(&self) -> Result<Option<usize>> {
		self.base.base.max_runtime_instances()
	}

	fn announce_block(&self) -> Result<bool> {
		self.base.base.announce_block()
	}

	fn telemetry_endpoints(
		&self,
		chain_spec: &Box<dyn ChainSpec>,
	) -> Result<Option<sc_telemetry::TelemetryEndpoints>> {
		self.base.base.telemetry_endpoints(chain_spec)
	}

	fn node_name(&self) -> Result<String> {
		self.base.base.node_name()
	}
}
