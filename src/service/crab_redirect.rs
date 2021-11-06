// This file is part of Darwinia.
//
// Copyright (C) 2018-2021 Darwinia Network
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
use std::{error::Error, sync::Arc};
// --- crates.io ---
use futures::lock::Mutex;
// --- paritytech ---
use cumulus_client_consensus_aura::{
	build_aura_consensus, BuildAuraConsensusParams, BuildVerifierParams, SlotProportion,
};
use cumulus_client_consensus_common::ParachainBlockImport;
use cumulus_client_consensus_relay_chain::{
	BuildRelayChainConsensusParams, Verifier as RelayChainVerifier,
};
use cumulus_primitives_core::ParaId;
use cumulus_primitives_parachain_inherent::ParachainInherentData;
use sc_basic_authorship::ProposerFactory;
use sc_client_api::ExecutorProvider;
use sc_consensus::{import_queue::BasicQueue, DefaultImportQueue};
use sc_executor::{NativeElseWasmExecutor, NativeExecutionDispatch, NativeVersion};
use sc_service::{error::Result, Configuration, TFullClient, TaskManager};
use sc_telemetry::TelemetryHandle;
use sp_consensus::{CanAuthorWithNativeVersion, SlotData};
use sp_consensus_aura::sr25519;
// --- darwinia-network ---
use super::*;
use crab_redirect_runtime::{api, RuntimeApi};
use darwinia_collator_primitives::OpaqueBlock as Block;

/// Native executor instance.
pub struct RuntimeExecutor;
impl NativeExecutionDispatch for RuntimeExecutor {
	type ExtendHostFunctions = ();

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		api::dispatch(method, data)
	}

	fn native_version() -> NativeVersion {
		crab_redirect_runtime::native_version()
	}
}

/// Build the import queue for the `Crab Redirect` runtime.
pub fn build_import_queue(
	client: Arc<TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<RuntimeExecutor>>>,
	config: &Configuration,
	telemetry: Option<TelemetryHandle>,
	task_manager: &TaskManager,
) -> Result<
	DefaultImportQueue<
		Block,
		TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<RuntimeExecutor>>,
	>,
> {
	let client2 = client.clone();

	let aura_verifier = move || {
		let slot_duration = cumulus_client_consensus_aura::slot_duration(&*client2).unwrap();

		Box::new(cumulus_client_consensus_aura::build_verifier::<
			sr25519::AuthorityPair,
			_,
			_,
			_,
		>(BuildVerifierParams {
			client: client2.clone(),
			create_inherent_data_providers: move |_, _| async move {
				let time = sp_timestamp::InherentDataProvider::from_system_time();

				let slot =
					sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
						*time,
						slot_duration.slot_duration(),
					);

				Ok((time, slot))
			},
			can_author_with: CanAuthorWithNativeVersion::new(client2.executor().clone()),
			telemetry,
		})) as Box<_>
	};

	let relay_chain_verifier = Box::new(RelayChainVerifier::new(client.clone(), |_, _| async {
		Ok(())
	})) as Box<_>;

	let verifier = Verifier {
		client: client.clone(),
		relay_chain_verifier,
		aura_verifier: BuildOnAccess::Uninitialized(Some(Box::new(aura_verifier))),
	};

	let registry = config.prometheus_registry().clone();
	let spawner = task_manager.spawn_essential_handle();

	Ok(BasicQueue::new(
		verifier,
		Box::new(ParachainBlockImport::new(client.clone())),
		None,
		&spawner,
		registry,
	))
}

/// Start a `Crab Redirect` parachain node.
pub async fn start_node(
	parachain_config: Configuration,
	polkadot_config: Configuration,
	id: ParaId,
) -> Result<(
	TaskManager,
	Arc<TFullClient<Block, RuntimeApi, NativeElseWasmExecutor<RuntimeExecutor>>>,
)> {
	start_node_impl::<RuntimeApi, RuntimeExecutor, _, _, _>(
		parachain_config,
		polkadot_config,
		id,
		|_| Ok(Default::default()),
		build_import_queue,
		|client,
		 prometheus_registry,
		 telemetry,
		 task_manager,
		 relay_chain_node,
		 transaction_pool,
		 sync_oracle,
		 keystore,
		 force_authoring| {
			let client2 = client.clone();
			let relay_chain_backend = relay_chain_node.backend.clone();
			let relay_chain_client = relay_chain_node.client.clone();
			let spawn_handle = task_manager.spawn_handle();
			let transaction_pool2 = transaction_pool.clone();
			let telemetry2 = telemetry.clone();
			let prometheus_registry2 = prometheus_registry.map(|r| (*r).clone());

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

				let relay_chain_backend2 = relay_chain_backend.clone();
				let relay_chain_client2 = relay_chain_client.clone();

				build_aura_consensus::<sr25519::AuthorityPair, _, _, _, _, _, _, _, _, _>(
					BuildAuraConsensusParams {
						proposer_factory,
						create_inherent_data_providers:
							move |_, (relay_parent, validation_data)| {
								let parachain_inherent =
									ParachainInherentData::create_at_with_client(
										relay_parent,
										&relay_chain_client,
										&*relay_chain_backend,
										&validation_data,
										id,
									);
								async move {
									let time =
										sp_timestamp::InherentDataProvider::from_system_time();

									let slot =
									sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
										*time,
										slot_duration.slot_duration(),
									);

									let parachain_inherent =
										parachain_inherent.ok_or_else(|| {
											Box::<dyn Error + Send + Sync>::from(
												"Failed to create parachain inherent",
											)
										})?;
									Ok((time, slot, parachain_inherent))
								}
							},
						block_import: client2.clone(),
						relay_chain_client: relay_chain_client2,
						relay_chain_backend: relay_chain_backend2,
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
					},
				)
			})));

			let proposer_factory = ProposerFactory::with_proof_recording(
				task_manager.spawn_handle(),
				client.clone(),
				transaction_pool,
				prometheus_registry.clone(),
				telemetry.clone(),
			);

			let relay_chain_backend = relay_chain_node.backend.clone();
			let relay_chain_client = relay_chain_node.client.clone();

			let relay_chain_consensus =
				cumulus_client_consensus_relay_chain::build_relay_chain_consensus(
					BuildRelayChainConsensusParams {
						para_id: id,
						proposer_factory,
						block_import: client.clone(),
						relay_chain_client: relay_chain_node.client.clone(),
						relay_chain_backend: relay_chain_node.backend.clone(),
						create_inherent_data_providers:
							move |_, (relay_parent, validation_data)| {
								let parachain_inherent =
									ParachainInherentData::create_at_with_client(
										relay_parent,
										&relay_chain_client,
										&*relay_chain_backend,
										&validation_data,
										id,
									);
								async move {
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
				client: client.clone(),
				aura_consensus: Arc::new(Mutex::new(aura_consensus)),
				relay_chain_consensus: Arc::new(Mutex::new(relay_chain_consensus)),
			});

			Ok(parachain_consensus)
		},
	)
	.await
}
