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

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

pub mod pallets;
pub use pallets::*;

pub mod weights;

// pub mod migrations;
// pub use migrations::*;

pub mod bridges_message;
pub use bridges_message::*;

pub mod wasm {
	//! Make the WASM binary available.

	include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

	/// Wasm binary unwrapped. If built with `BUILD_DUMMY_WASM_BINARY`, the function panics.
	#[cfg(feature = "std")]
	pub fn wasm_binary_unwrap() -> &'static [u8] {
		WASM_BINARY.expect(
			"Development wasm binary is not available. This means the client is \
			built with `BUILD_DUMMY_WASM_BINARY` flag and it is only usable for \
			production chains. Please rebuild with the flag disabled.",
		)
	}
}
pub use wasm::*;

#[cfg(feature = "runtime-benchmarks")]
#[macro_use]
extern crate frame_benchmarking;
#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(
		[frame_system, SystemBench::<Runtime>]
		[pallet_timestamp, Timestamp]
		[pallet_balances, Balances]
		[pallet_collator_selection, CollatorSelection]
		// TODO wait for https://github.com/paritytech/substrate/issues/11068
		// [pallet_session, SessionBench::<Runtime>]
		[pallet_utility, Utility]
		[pallet_multisig, Multisig]
		[pallet_proxy, Proxy]
		[cumulus_pallet_xcmp_queue, XcmpQueue]
	);
}

pub use dc_primitives::*;

// --- paritytech ---
use sp_core::OpaqueMetadata;
use sp_runtime::{
	generic,
	traits::Block as BlockT,
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
// --- darwinia-network ---
use dp_common_runtime::*;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
>;

type Ring = Balances;

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: sp_runtime::create_runtime_str!("Darwinia Parachain"),
	impl_name: sp_runtime::create_runtime_str!("Darwinia Parachain"),
	authoring_version: 1,
	spec_version: 5_3_7_0,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 0,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

frame_support::construct_runtime! {
	pub enum Runtime
	where
		Block = Block,
		NodeBlock = OpaqueBlock,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 0,
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Inherent, Storage, Config, Event<T>, ValidateUnsigned} = 1,
		Timestamp: pallet_timestamp::{Pallet, Call, Inherent, Storage} = 3,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 4,

		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 5,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage, Event<T>} = 6,

		// Collator things. the order of these 4 are important and shall not change.
		Authorship: pallet_authorship::{Pallet, Call, Storage} = 7,
		CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 8,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 9,
		Aura: pallet_aura::{Pallet, Storage, Config<T>} = 10,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config} = 11,
		RemoteGovernance: dp_common_runtime::remote_governance::{Pallet, Call, Storage, Event<T>} = 25,

		// XCM things.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 12,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Storage, Event<T>, Origin, Config} = 13,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 14,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 15,

		Utility: pallet_utility::{Pallet, Call, Event} = 16,
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 17,
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 18,
		Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>} = 19,

		// Darwinia Parachain <> Darwinia.
		BridgeDarwiniaGrandpa: pallet_bridge_grandpa::<Instance1>::{Pallet, Call, Storage} = 20,
		BridgeDarwiniaMessages: pallet_bridge_messages::<Instance1>::{Pallet, Call, Storage, Event<T>} = 21,
		BridgeDarwiniaDispatch: pallet_bridge_dispatch::<Instance1>::{Pallet, Event<T>} = 22,

		DarwiniaFeeMarket: pallet_fee_market::<Instance1>::{Pallet, Call, Storage, Event<T>} = 23,
		FromDarwiniaIssuing: dp_common_runtime::helixbridge::{Pallet, Call, Storage, Event<T>} = 24,

		MessageRouter: dp_common_runtime::message_router::{Pallet, Call, Storage, Event<T>} = 26,
		SoloToPara: cumulus_pallet_solo_to_para::{Pallet, Call, Storage, Event} = 27,
	}
}

sp_api::impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(
			extrinsic: <Block as BlockT>::Extrinsic,
		) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(block: Block, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}

		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			Aura::authorities().into_inner()
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl cumulus_primitives_core::CollectCollationInfo<Block> for Runtime {
		fn collect_collation_info(header: &<Block as BlockT>::Header) -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info(header)
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;


			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();
			return (list, storage_info)
		}
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, TrackedStorageKey};
			use frame_system_benchmarking::Pallet as SystemBench;

			impl frame_system_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				array_bytes::hex_into_unchecked("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac"),
				// Total Issuance
				array_bytes::hex_into_unchecked("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80"),
				// Execution Phase
				array_bytes::hex_into_unchecked("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a"),
				// Event Count
				array_bytes::hex_into_unchecked("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850"),
				// System Events
				array_bytes::hex_into_unchecked("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7"),
				// Configuration ActiveConfig
				array_bytes::hex_into_unchecked("06de3d8a54d27e44a9d5ce189618f22db4b49d95320d9021994c850f25b8e385"),
			];

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			Ok(batches)
		}
	}
}

struct CheckInherents;
impl cumulus_pallet_parachain_system::CheckInherents<Block> for CheckInherents {
	fn check_inherents(
		block: &Block,
		relay_state_proof: &cumulus_pallet_parachain_system::RelayChainStateProof,
	) -> sp_inherents::CheckInherentsResult {
		let relay_chain_slot = relay_state_proof
			.read_slot()
			.expect("Could not read the relay chain slot from the proof");

		let inherent_data =
			cumulus_primitives_timestamp::InherentDataProvider::from_relay_chain_slot_and_duration(
				relay_chain_slot,
				sp_std::time::Duration::from_secs(6),
			)
			.create_inherent_data()
			.expect("Could not create the timestamp inherent data");

		inherent_data.check_extrinsics(block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}
