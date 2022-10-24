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

pub mod api;
pub use api::*;

pub mod executors;
pub use executors::*;

pub use crab_parachain_runtime::RuntimeApi as CrabParachainRuntimeApi;
pub use darwinia_parachain_runtime::RuntimeApi as DarwiniaParachainRuntimeApi;
pub use pangolin_parachain_runtime::RuntimeApi as PangolinParachainRuntimeApi;

// --- std ---
use std::{error::Error, sync::Arc, time::Duration};
// --- crates.io ---
use futures::lock::Mutex;
use jsonrpsee::RpcModule;
// --- paritytech ---
use cumulus_client_cli::CollatorOptions;
use cumulus_client_consensus_aura::{
	AuraConsensus, BuildAuraConsensusParams, BuildVerifierParams, SlotProportion,
};
use cumulus_client_consensus_common::{
	ParachainBlockImport, ParachainCandidate, ParachainConsensus,
};
use cumulus_client_consensus_relay_chain::{
	BuildRelayChainConsensusParams, Verifier as RelayChainVerifier,
};
use cumulus_client_network::BlockAnnounceValidator;
use cumulus_client_service::{
	prepare_node_config, start_collator, start_full_node, StartCollatorParams, StartFullNodeParams,
};
use cumulus_primitives_core::{
	relay_chain::v2::{Hash as PHash, PersistedValidationData},
	ParaId,
};
use cumulus_primitives_parachain_inherent::ParachainInherentData;
use cumulus_relay_chain_inprocess_interface::build_inprocess_relay_chain;
use cumulus_relay_chain_interface::{RelayChainError, RelayChainInterface, RelayChainResult};
use cumulus_relay_chain_rpc_interface::RelayChainRPCInterface;
use polkadot_service::CollatorPair;
use sc_basic_authorship::ProposerFactory;
use sc_client_api::StateBackendFor;
use sc_consensus::{
	import_queue::{BasicQueue, Verifier as VerifierT},
	BlockImportParams, DefaultImportQueue,
};
use sc_executor::WasmExecutor;
use sc_network::NetworkService;
use sc_service::{
	error::Result, ChainSpec, Configuration, PartialComponents, TFullBackend, TFullClient,
	TaskManager,
};
use sc_telemetry::{Telemetry, TelemetryHandle, TelemetryWorker, TelemetryWorkerHandle};
use sc_transaction_pool::FullPool;
use sp_api::{ApiExt, HeaderT};
use sp_consensus::{AlwaysCanAuthor, CacheKeyId};
use sp_consensus_aura::sr25519::{AuthorityId as AuraId, AuthorityPair as AuraPair};
use sp_keystore::SyncCryptoStorePtr;
use sp_runtime::{generic::BlockId, traits::BlakeTwo256};
use substrate_prometheus_endpoint::Registry;
// --- darwinia-network ---
use dc_primitives::{OpaqueBlock as Block, *};
use dc_rpc::FullDeps;

type FullBackend = TFullBackend<Block>;
type FullClient<RuntimeApi> = TFullClient<Block, RuntimeApi, WasmExecutor<HostFunctions>>;
type StateBackend = StateBackendFor<FullBackend, Block>;

/// Can be called for a `Configuration` to check if it is a configuration for the `Crab Parachain`
/// network.
pub trait IdentifyVariant {
	/// Returns if this is a configuration for the `Crab Parachain` network.
	fn is_crab_parachain(&self) -> bool;

	/// Returns if this is a configuration for the `Pangolin Parachain` network.
	fn is_pangolin_parachain(&self) -> bool;

	/// Returns true if this configuration is for a development network.
	fn is_dev(&self) -> bool;
}
impl IdentifyVariant for Box<dyn ChainSpec> {
	fn is_crab_parachain(&self) -> bool {
		self.id().starts_with("crab_parachain")
	}

	fn is_pangolin_parachain(&self) -> bool {
		self.id().starts_with("pangolin_parachain")
	}

	fn is_dev(&self) -> bool {
		self.id().ends_with("dev")
	}
}

enum BuildOnAccess<R> {
	Uninitialized(Option<Box<dyn FnOnce() -> R + Send + Sync>>),
	Initialized(R),
}
impl<R> BuildOnAccess<R> {
	fn get_mut(&mut self) -> &mut R {
		loop {
			match self {
				Self::Uninitialized(f) => {
					*self = Self::Initialized((f.take().unwrap())());
				},
				Self::Initialized(ref mut r) => return r,
			}
		}
	}
}

struct Verifier<Client> {
	client: Arc<Client>,
	aura_verifier: BuildOnAccess<Box<dyn VerifierT<Block>>>,
	relay_chain_verifier: Box<dyn VerifierT<Block>>,
}
#[async_trait::async_trait]
impl<Client> VerifierT<Block> for Verifier<Client>
where
	Client: sp_api::ProvideRuntimeApi<Block> + Send + Sync,
	Client::Api: sp_consensus_aura::AuraApi<Block, AuraId>,
{
	async fn verify(
		&mut self,
		block_import: BlockImportParams<Block, ()>,
	) -> std::result::Result<
		(BlockImportParams<Block, ()>, Option<Vec<(CacheKeyId, Vec<u8>)>>),
		String,
	> {
		let block_id = BlockId::hash(*block_import.header.parent_hash());

		if self
			.client
			.runtime_api()
			.has_api::<dyn sp_consensus_aura::AuraApi<Block, AuraId>>(&block_id)
			.unwrap_or(false)
		{
			self.aura_verifier.get_mut().verify(block_import).await
		} else {
			self.relay_chain_verifier.verify(block_import).await
		}
	}
}

/// Special [`ParachainConsensus`] implementation that waits for the upgrade from
/// shell to a parachain runtime that implements Aura.
struct WaitForAuraConsensus<Client> {
	client: Arc<Client>,
	aura_consensus: Arc<Mutex<BuildOnAccess<Box<dyn ParachainConsensus<Block>>>>>,
	relay_chain_consensus: Arc<Mutex<Box<dyn ParachainConsensus<Block>>>>,
}
impl<Client> Clone for WaitForAuraConsensus<Client> {
	fn clone(&self) -> Self {
		Self {
			client: self.client.clone(),
			aura_consensus: self.aura_consensus.clone(),
			relay_chain_consensus: self.relay_chain_consensus.clone(),
		}
	}
}
#[async_trait::async_trait]
impl<Client> ParachainConsensus<Block> for WaitForAuraConsensus<Client>
where
	Client: sp_api::ProvideRuntimeApi<Block> + Send + Sync,
	Client::Api: sp_consensus_aura::AuraApi<Block, AuraId>,
{
	async fn produce_candidate(
		&mut self,
		parent: &Header,
		relay_parent: PHash,
		validation_data: &PersistedValidationData,
	) -> Option<ParachainCandidate<Block>> {
		let block_id = BlockId::hash(parent.hash());
		if self
			.client
			.runtime_api()
			.has_api::<dyn sp_consensus_aura::AuraApi<Block, AuraId>>(&block_id)
			.unwrap_or(false)
		{
			self.aura_consensus
				.lock()
				.await
				.get_mut()
				.produce_candidate(parent, relay_parent, validation_data)
				.await
		} else {
			self.relay_chain_consensus
				.lock()
				.await
				.produce_candidate(parent, relay_parent, validation_data)
				.await
		}
	}
}

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
#[allow(clippy::type_complexity)]
pub fn new_partial<RuntimeApi, BIQ>(
	config: &Configuration,
	build_import_queue: BIQ,
) -> Result<
	PartialComponents<
		FullClient<RuntimeApi>,
		FullBackend,
		(),
		DefaultImportQueue<Block, FullClient<RuntimeApi>>,
		FullPool<Block, FullClient<RuntimeApi>>,
		(Option<Telemetry>, Option<TelemetryWorkerHandle>),
	>,
>
where
	RuntimeApi: 'static + Send + Sync + sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi>>,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	BIQ: FnOnce(
		Arc<FullClient<RuntimeApi>>,
		&Configuration,
		Option<TelemetryHandle>,
		&TaskManager,
	) -> Result<DefaultImportQueue<Block, FullClient<RuntimeApi>>>,
	StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> std::result::Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;
	let executor = sc_executor::WasmExecutor::<HostFunctions>::new(
		config.wasm_method,
		config.default_heap_pages,
		config.max_runtime_instances,
		None,
		config.runtime_cache_size,
	);
	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;
	let client = Arc::new(client);
	let telemetry_worker_handle = telemetry.as_ref().map(|(worker, _)| worker.handle());
	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});
	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);
	let import_queue = build_import_queue(
		client.clone(),
		config,
		telemetry.as_ref().map(|telemetry| telemetry.handle()),
		&task_manager,
	)?;
	let params = PartialComponents {
		backend,
		client,
		import_queue,
		keystore_container,
		task_manager,
		transaction_pool,
		select_chain: (),
		other: (telemetry, telemetry_worker_handle),
	};

	Ok(params)
}

async fn build_relay_chain_interface(
	polkadot_config: Configuration,
	parachain_config: &Configuration,
	telemetry_worker_handle: Option<TelemetryWorkerHandle>,
	task_manager: &mut TaskManager,
	collator_options: CollatorOptions,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> RelayChainResult<(Arc<(dyn RelayChainInterface + 'static)>, Option<CollatorPair>)> {
	match collator_options.relay_chain_rpc_url {
		Some(relay_chain_url) =>
			Ok((Arc::new(RelayChainRPCInterface::new(relay_chain_url).await?) as Arc<_>, None)),
		None => build_inprocess_relay_chain(
			polkadot_config,
			parachain_config,
			telemetry_worker_handle,
			task_manager,
			hwbench,
		),
	}
}

/// Start a node with the given parachain `Configuration` and relay chain `Configuration`.
///
/// This is the actual implementation that is abstract over the executor and the runtime api.
#[allow(clippy::too_many_arguments)]
#[sc_tracing::logging::prefix_logs_with("Parachain")]
async fn start_node_impl<RuntimeApi, RB, BIQ, BIC>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	id: ParaId,
	_rpc_ext_builder: RB,
	build_import_queue: BIQ,
	build_consensus: BIC,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> Result<(TaskManager, Arc<FullClient<RuntimeApi>>)>
where
	RuntimeApi: 'static + Send + Sync + sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi>>,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	RB: 'static + Send + Fn(Arc<FullClient<RuntimeApi>>) -> Result<RpcModule<()>>,
	BIQ: FnOnce(
		Arc<FullClient<RuntimeApi>>,
		&Configuration,
		Option<TelemetryHandle>,
		&TaskManager,
	) -> Result<DefaultImportQueue<Block, FullClient<RuntimeApi>>>,
	BIC: FnOnce(
		Arc<FullClient<RuntimeApi>>,
		Option<&Registry>,
		Option<TelemetryHandle>,
		&TaskManager,
		Arc<dyn RelayChainInterface>,
		Arc<FullPool<Block, FullClient<RuntimeApi>>>,
		Arc<NetworkService<Block, Hash>>,
		SyncCryptoStorePtr,
		bool,
	) -> Result<Box<dyn ParachainConsensus<Block>>>,
	StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
	let parachain_config = prepare_node_config(parachain_config);
	let params = new_partial::<RuntimeApi, BIQ>(&parachain_config, build_import_queue)?;
	let (mut telemetry, telemetry_worker_handle) = params.other;
	let client = params.client.clone();
	let backend = params.backend.clone();
	let mut task_manager = params.task_manager;
	let (relay_chain_interface, collator_key) = build_relay_chain_interface(
		polkadot_config,
		&parachain_config,
		telemetry_worker_handle,
		&mut task_manager,
		collator_options.clone(),
		hwbench.clone(),
	)
	.await
	.map_err(|e| match e {
		RelayChainError::ServiceError(polkadot_service::Error::Sub(x)) => x,
		s => s.to_string().into(),
	})?;
	let block_announce_validator = BlockAnnounceValidator::new(relay_chain_interface.clone(), id);
	let force_authoring = parachain_config.force_authoring;
	let validator = parachain_config.role.is_authority();
	let prometheus_registry = parachain_config.prometheus_registry().cloned();
	let transaction_pool = params.transaction_pool.clone();
	let import_queue = cumulus_client_service::SharedImportQueue::new(params.import_queue);
	let (network, system_rpc_tx, start_network) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &parachain_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue: import_queue.clone(),
			block_announce_validator_builder: Some(Box::new(|_| {
				Box::new(block_announce_validator)
			})),
			warp_sync: None,
		})?;
	let rpc_builder = {
		let client = client.clone();
		let transaction_pool = transaction_pool.clone();

		Box::new(move |deny_unsafe, _| {
			let deps =
				FullDeps { client: client.clone(), pool: transaction_pool.clone(), deny_unsafe };

			dc_rpc::create_full(deps).map_err(Into::into)
		})
	};

	sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		rpc_builder,
		client: client.clone(),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		config: parachain_config,
		keystore: params.keystore_container.sync_keystore(),
		backend: backend.clone(),
		network: network.clone(),
		system_rpc_tx,
		telemetry: telemetry.as_mut(),
	})?;

	if let Some(hwbench) = hwbench {
		sc_sysinfo::print_hwbench(&hwbench);

		if let Some(ref mut telemetry) = telemetry {
			let telemetry_handle = telemetry.handle();
			task_manager.spawn_handle().spawn(
				"telemetry_hwbench",
				None,
				sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
			);
		}
	}

	let announce_block = {
		let network = network.clone();
		Arc::new(move |hash, data| network.announce_block(hash, data))
	};
	let relay_chain_slot_duration = Duration::from_secs(6);

	if validator {
		let parachain_consensus = build_consensus(
			client.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|t| t.handle()),
			&task_manager,
			relay_chain_interface.clone(),
			transaction_pool,
			network,
			params.keystore_container.sync_keystore(),
			force_authoring,
		)?;
		let spawner = task_manager.spawn_handle();
		let params = StartCollatorParams {
			para_id: id,
			block_status: client.clone(),
			announce_block,
			client: client.clone(),
			task_manager: &mut task_manager,
			relay_chain_interface,
			spawner,
			parachain_consensus,
			import_queue,
			collator_key: collator_key.expect("Command line arguments do not allow this. qed"),
			relay_chain_slot_duration,
		};

		start_collator(params).await?;
	} else {
		let params = StartFullNodeParams {
			client: client.clone(),
			announce_block,
			task_manager: &mut task_manager,
			para_id: id,
			relay_chain_interface,
			relay_chain_slot_duration,
			import_queue,
			collator_options,
		};

		start_full_node(params)?;
	}

	start_network.start_network();

	Ok((task_manager, client))
}

/// Build the import queue for the darwinia-parachain.
pub fn build_import_queue<RuntimeApi>(
	client: Arc<FullClient<RuntimeApi>>,
	config: &Configuration,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
) -> Result<DefaultImportQueue<Block, FullClient<RuntimeApi>>>
where
	RuntimeApi: 'static + Send + Sync + sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi>>,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
{
	let client2 = client.clone();

	let aura_verifier = move || {
		let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client2).unwrap();

		Box::new(cumulus_client_consensus_aura::build_verifier::<AuraPair, _, _, _>(
			BuildVerifierParams {
				client: client2.clone(),
				create_inherent_data_providers: move |_, _| async move {
					let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

					let slot =
					sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
						*timestamp,
						slot_duration,
					);

					Ok((timestamp, slot))
				},
				can_author_with: AlwaysCanAuthor,
				telemetry,
			},
		)) as Box<_>
	};

	let relay_chain_verifier =
		Box::new(RelayChainVerifier::new(client.clone(), |_, _| async { Ok(()) })) as Box<_>;

	let verifier = Verifier {
		client: client.clone(),
		relay_chain_verifier,
		aura_verifier: BuildOnAccess::Uninitialized(Some(Box::new(aura_verifier))),
	};

	let registry = config.prometheus_registry();
	let spawner = task_manager.spawn_essential_handle();

	Ok(BasicQueue::new(
		verifier,
		Box::new(ParachainBlockImport::new(client)),
		None,
		&spawner,
		registry,
	))
}

/// Start a darwinia-parachain node.
pub async fn start_node<RuntimeApi>(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	collator_options: CollatorOptions,
	id: ParaId,
	hwbench: Option<sc_sysinfo::HwBench>,
) -> Result<(TaskManager, Arc<FullClient<RuntimeApi>>)>
where
	RuntimeApi: 'static + Send + Sync + sp_api::ConstructRuntimeApi<Block, FullClient<RuntimeApi>>,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
{
	start_node_impl::<RuntimeApi, _, _, _>(
		parachain_config,
		polkadot_config,
		collator_options,
		id,
		|_| Ok(RpcModule::new(())),
		build_import_queue,
		|client,
		 prometheus_registry,
		 telemetry,
		 task_manager,
		 relay_chain_interface,
		 transaction_pool,
		 sync_oracle,
		 keystore,
		 force_authoring| {
			let client2 = client.clone();
			let spawn_handle = task_manager.spawn_handle();
			let transaction_pool2 = transaction_pool.clone();
			let telemetry2 = telemetry.clone();
			let prometheus_registry2 = prometheus_registry.map(|r| (*r).clone());
			let relay_chain_for_aura = relay_chain_interface.clone();
			let aura_consensus = BuildOnAccess::Uninitialized(Some(Box::new(move || {
				let slot_duration =
					cumulus_client_consensus_aura::slot_duration(&*client2).unwrap();
				let proposer_factory = ProposerFactory::with_proof_recording(
					spawn_handle,
					client2.clone(),
					transaction_pool2,
					prometheus_registry2.as_ref(),
					telemetry2.clone(),
				);

				AuraConsensus::build::<AuraPair, _, _, _, _, _, _>(BuildAuraConsensusParams {
					proposer_factory,
					create_inherent_data_providers: move |_, (relay_parent, validation_data)| {
						let relay_chain_for_aura = relay_chain_for_aura.clone();

						async move {
							let parachain_inherent = ParachainInherentData::create_at(
								relay_parent,
								&relay_chain_for_aura,
								&validation_data,
								id,
							)
							.await;
							let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

							let slot =
									sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
										*timestamp,
										slot_duration,
									);

							let parachain_inherent = parachain_inherent.ok_or_else(|| {
								Box::<dyn Error + Send + Sync>::from(
									"Failed to create parachain inherent",
								)
							})?;
							Ok((timestamp, slot, parachain_inherent))
						}
					},
					block_import: client2.clone(),
					para_client: client2.clone(),
					backoff_authoring_blocks: Option::<()>::None,
					sync_oracle,
					keystore,
					force_authoring,
					slot_duration,
					// We got around 500ms for proposing
					block_proposal_slot_portion: SlotProportion::new(1f32 / 24f32),
					// And a maximum of 750ms if slots are skipped
					max_block_proposal_slot_portion: Some(SlotProportion::new(1f32 / 16f32)),
					telemetry: telemetry2,
				})
			})));
			let proposer_factory = ProposerFactory::with_proof_recording(
				task_manager.spawn_handle(),
				client.clone(),
				transaction_pool,
				prometheus_registry,
				telemetry,
			);
			let relay_chain_consensus =
				cumulus_client_consensus_relay_chain::build_relay_chain_consensus(
					BuildRelayChainConsensusParams {
						para_id: id,
						proposer_factory,
						block_import: client.clone(),
						relay_chain_interface: relay_chain_interface.clone(),
						create_inherent_data_providers:
							move |_, (relay_parent, validation_data)| {
								let relay_chain_interface = relay_chain_interface.clone();

								async move {
									let parachain_inherent = ParachainInherentData::create_at(
										relay_parent,
										&relay_chain_interface,
										&validation_data,
										id,
									)
									.await;
									let parachain_inherent =
										parachain_inherent.ok_or_else(|| {
											Box::<dyn Error + Send + Sync>::from(
												"Failed to create parachain inherent",
											)
										})?;
									Ok(parachain_inherent)
								}
							},
					},
				);

			let parachain_consensus = Box::new(WaitForAuraConsensus {
				client,
				aura_consensus: Arc::new(Mutex::new(aura_consensus)),
				relay_chain_consensus: Arc::new(Mutex::new(relay_chain_consensus)),
			});

			Ok(parachain_consensus)
		},
		hwbench,
	)
	.await
}
