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

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

pub mod pallets;
pub use pallets::*;

pub mod bridges_message;
pub use bridges_message::*;

pub mod weights;

pub mod wasm {
	//! Make the WASM binary available.

	include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

	/// Wasm binary unwrapped. If built with `BUILD_DUMMY_WASM_BINARY`, the function panics.
	#[cfg(feature = "std")]
	pub fn wasm_binary_unwrap() -> &'static [u8] {
		return WASM_BINARY.expect(
			"Development wasm binary is not available. This means the client is \
			built with `BUILD_DUMMY_WASM_BINARY` flag and it is only usable for \
			production chains. Please rebuild with the flag disabled.",
		);
	}
}
pub use wasm::*;

mod migrations;
use migrations::*;

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
		// TODO: wait for https://github.com/paritytech/substrate/issues/11068
		// [pallet_session, SessionBench::<Runtime>]
		[pallet_utility, Utility]
		[pallet_multisig, Multisig]
		[pallet_proxy, Proxy]
		[pallet_bridge_grandpa, BridgePangolinGrandpa]
		// TODO: https://github.com/darwinia-network/darwinia-parachain/issues/66
		// [pallet_bridge_messages, MessagesBench::<Runtime, WithPangolinMessages>]
		[pallet_fee_market, PangolinFeeMarket]
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
use dc_common_runtime::*;

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
	CustomOnRuntimeUpgrade,
>;

type Ring = Balances;

/// This runtime version.
#[cfg(not(feature = "alpha"))]
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: sp_runtime::create_runtime_str!("Pangolin Parachain"),
	impl_name: sp_runtime::create_runtime_str!("Pangolin Parachain"),
	authoring_version: 1,
	spec_version: 5,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 0,
};

/// This runtime version.
#[cfg(feature = "alpha")]
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: sp_runtime::create_runtime_str!("Pangolin Parachain Alpha"),
	impl_name: sp_runtime::create_runtime_str!("Pangolin Parachain Alpha"),
	authoring_version: 1,
	spec_version: 5,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 0,
};

frame_support::construct_runtime! {
	pub enum Runtime
	where
		Block = Block,
		NodeBlock = OpaqueBlock,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		// System support stuff.
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>} = 0,
		ParachainSystem: cumulus_pallet_parachain_system::{Pallet, Call, Inherent, Storage, Config, Event<T>, ValidateUnsigned} = 1,
		Timestamp: pallet_timestamp::{Pallet, Call, Inherent, Storage} = 3,
		ParachainInfo: parachain_info::{Pallet, Storage, Config} = 4,

		// Monetary stuff.
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 5,
		TransactionPayment: pallet_transaction_payment::{Pallet, Storage} = 6,

		// Collator support. the order of these 4 are important and shall not change.
		Authorship: pallet_authorship::{Pallet, Call, Storage} = 7,
		CollatorSelection: pallet_collator_selection::{Pallet, Call, Storage, Event<T>, Config<T>} = 8,
		Session: pallet_session::{Pallet, Call, Storage, Event, Config<T>} = 9,
		Aura: pallet_aura::{Pallet, Storage, Config<T>} = 10,
		AuraExt: cumulus_pallet_aura_ext::{Pallet, Storage, Config} = 11,

		// XCM helpers.
		XcmpQueue: cumulus_pallet_xcmp_queue::{Pallet, Call, Storage, Event<T>} = 12,
		PolkadotXcm: pallet_xcm::{Pallet, Call, Storage, Event<T>, Origin, Config} = 13,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 14,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 15,

		// Handy utilities.
		Utility: pallet_utility::{Pallet, Call, Event} = 16,
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 17,
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 18,
		Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>} = 19,

		// S2S bridges.
		BridgePangolinGrandpa: pallet_bridge_grandpa::<Instance1>::{Pallet, Call, Storage} = 20,
		BridgePangolinMessages: pallet_bridge_messages::<Instance1>::{Pallet, Call, Storage, Event<T>} = 21,
		BridgePangolinDispatch: pallet_bridge_dispatch::<Instance1>::{Pallet, Event<T>} = 22,

		PangolinFeeMarket: pallet_fee_market::<Instance1>::{Pallet, Call, Storage, Event<T>} = 23,
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

	impl bp_pangolin::PangolinFinalityApi<Block> for Runtime {
		fn best_finalized() -> (bp_pangolin::BlockNumber, bp_pangolin::Hash) {
			let header = BridgePangolinGrandpa::best_finalized();
			(header.number, header.hash())
		}
	}

	impl bp_pangolin::ToPangolinOutboundLaneApi<Block, Balance, bm_pangolin::ToPangolinMessagePayload> for Runtime {
		fn message_details(
			lane: bp_messages::LaneId,
			begin: bp_messages::MessageNonce,
			end: bp_messages::MessageNonce,
		) -> Vec<bp_messages::MessageDetails<Balance>> {
			bridge_runtime_common::messages_api::outbound_message_details::<
				Runtime,
				WithPangolinMessages,
				bm_pangolin::WithPangolinMessageBridge,
			>(lane, begin, end)
		}

		fn latest_received_nonce(lane: bp_messages::LaneId) -> bp_messages::MessageNonce {
			BridgePangolinMessages::outbound_latest_received_nonce(lane)
		}

		fn latest_generated_nonce(lane: bp_messages::LaneId) -> bp_messages::MessageNonce {
			BridgePangolinMessages::outbound_latest_generated_nonce(lane)
		}
	}

	impl bp_pangolin::FromPangolinInboundLaneApi<Block> for Runtime {
		fn latest_received_nonce(lane: bp_messages::LaneId) -> bp_messages::MessageNonce {
			BridgePangolinMessages::inbound_latest_received_nonce(lane)
		}

		fn latest_confirmed_nonce(lane: bp_messages::LaneId) -> bp_messages::MessageNonce {
			BridgePangolinMessages::inbound_latest_confirmed_nonce(lane)
		}

		fn unrewarded_relayers_state(lane: bp_messages::LaneId) -> bp_messages::UnrewardedRelayersState {
			BridgePangolinMessages::inbound_unrewarded_relayers_state(lane)
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
			use pallet_bridge_messages::benchmarking::Pallet as MessagesBench;

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
			use frame_support::assert_ok;
			use bridge_runtime_common::messages;
			use pallet_bridge_messages::benchmarking::{
				Pallet as MessagesBench,
				Config as MessagesConfig,
				MessageDeliveryProofParams,
				MessageParams,
				MessageProofParams,
			};
			use bridges_message::pangolin::{
				WithPangolinMessageBridge,
				FromPangolinMessagesProof,
				ToPangolinMessagePayload,
				ToPangolinMessagesDeliveryProof
			};
			use bridge_runtime_common::{
				messages_benchmarking::{
					prepare_message_proof,
					prepare_message_delivery_proof
				},
				messages::{
					source:: FromThisChainMessagePayload
				}
			};
			use bp_runtime::{messages::DispatchFeePayment};
			use pallet_fee_market::Instance1;

			impl frame_system_benchmarking::Config for Runtime {}
			impl MessagesConfig<WithPangolinMessages> for Runtime {
				fn maximal_message_size() -> u32 {
					messages::source::maximal_message_size::<WithPangolinMessageBridge>()
				}

				fn bridged_relayer_id() -> Self::InboundRelayer {
					[0u8; 32].into()
				}

				fn account_balance(account: &Self::AccountId) -> Self::OutboundMessageFee {
					pallet_balances::Pallet::<Runtime>::free_balance(account)
				}

				fn endow_account(account: &Self::AccountId) {
					// --- paritytech ---
					use frame_support::traits::Currency;

					// prepare fee_market
					let collateral = <Runtime as pallet_fee_market::Config<Instance1>>::CollateralPerOrder::get();
					let caller1: <Runtime as frame_system::Config>::AccountId = frame_benchmarking::account("source", 1, 0u32);
					let caller2: <Runtime as frame_system::Config>::AccountId = frame_benchmarking::account("source", 2, 0u32);
					let caller3: <Runtime as frame_system::Config>::AccountId = frame_benchmarking::account("source", 3, 0u32);

					<Runtime as pallet_fee_market::Config<Instance1>>::Currency::make_free_balance_be(&caller1, collateral.saturating_mul(10u32.into()));
					<Runtime as pallet_fee_market::Config<Instance1>>::Currency::make_free_balance_be(&caller2, collateral.saturating_mul(10u32.into()));
					<Runtime as pallet_fee_market::Config<Instance1>>::Currency::make_free_balance_be(&caller3, collateral.saturating_mul(10u32.into()));

					assert_ok!(pallet_fee_market::Pallet::<Runtime, Instance1>::enroll_and_lock_collateral(
						frame_system::RawOrigin::Signed(caller1).into(),
						collateral,
						None
					));
					assert_ok!(pallet_fee_market::Pallet::<Runtime, Instance1>::enroll_and_lock_collateral(
						frame_system::RawOrigin::Signed(caller2).into(),
						collateral,
						None
					));
					assert_ok!(pallet_fee_market::Pallet::<Runtime, Instance1>::enroll_and_lock_collateral(
						frame_system::RawOrigin::Signed(caller3).into(),
						collateral,
						None
					));
					// prepare current account
					pallet_balances::Pallet::<Runtime>::make_free_balance_be(
						account,
						Balance::MAX/100,
					);
				}

				fn prepare_outbound_message(
					params: MessageParams<Self::AccountId>,
				) -> (ToPangolinMessagePayload, Balance) {
					let message_payload = vec![0; params.size as usize];
					let dispatch_origin = bp_message_dispatch::CallOrigin::SourceAccount(params.sender_account);
					let message = FromThisChainMessagePayload::<WithPangolinMessageBridge> {
						spec_version: 0,
						weight: params.size as _,
						origin: dispatch_origin,
						call: message_payload,
						dispatch_fee_payment: DispatchFeePayment::AtSourceChain,
					};

					(message, <Runtime as pallet_fee_market::Config<Instance1>>::MinimumRelayFee::get())
				}

				fn prepare_message_proof(
					params: MessageProofParams,
				) -> (FromPangolinMessagesProof, frame_support::weights::Weight) {
					prepare_message_proof::<Runtime, (), WithPangolinGrandpa, WithPangolinMessageBridge, bp_pangolin::Header, bp_polkadot_core::Hasher>(
						params,
						&VERSION,
						Balance::MAX / 100,
					)
				}

				fn prepare_message_delivery_proof(
					params: MessageDeliveryProofParams<Self::AccountId>,
				) -> ToPangolinMessagesDeliveryProof {
					prepare_message_delivery_proof::<Runtime, WithPangolinGrandpa, WithPangolinMessageBridge, bp_pangolin::Header, bp_polkadot_core::Hasher>(
						params,
					)
				}

				fn is_message_dispatched(nonce: bp_messages::MessageNonce) -> bool {
					frame_system::Pallet::<Runtime>::events()
						.into_iter()
						.map(|event_record| event_record.event)
						.any(|event| matches!(
							event,
							Event::BridgePangolinDispatch(pallet_bridge_dispatch::Event::<Runtime, _>::MessageDispatched(
								_, ([0, 0, 0, 0], nonce_from_event), _,
							)) if nonce_from_event == nonce
						))
				}
			}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				array_bytes::hex2bytes_unchecked("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").into(),
				// Total Issuance
				array_bytes::hex2bytes_unchecked("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").into(),
				// Execution Phase
				array_bytes::hex2bytes_unchecked("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").into(),
				// Event Count
				array_bytes::hex2bytes_unchecked("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").into(),
				// System Events
				array_bytes::hex2bytes_unchecked("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").into(),
				// Treasury Account
				array_bytes::hex2bytes_unchecked("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000").into(),
				// Caller 0 Account
				array_bytes::hex2bytes_unchecked("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da946c154ffd9992e395af90b5b13cc6f295c77033fce8a9045824a6690bbf99c6db269502f0a8d1d2a008542d5690a0749").into(),
				// Configuration ActiveConfig
				array_bytes::hex2bytes_unchecked("06de3d8a54d27e44a9d5ce189618f22db4b49d95320d9021994c850f25b8e385").into(),
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

		inherent_data.check_extrinsics(&block)
	}
}

cumulus_pallet_parachain_system::register_validate_block! {
	Runtime = Runtime,
	BlockExecutor = cumulus_pallet_aura_ext::BlockExecutor::<Runtime, Executive>,
	CheckInherents = CheckInherents,
}

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}
