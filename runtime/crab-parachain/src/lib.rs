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

pub mod constants {
	pub mod currency {
		// --- darwinia-network ---
		use darwinia_collator_primitives::*;

		pub const NANO: Balance = 1;
		pub const MICRO: Balance = 1_000 * NANO;
		pub const MILLI: Balance = 1_000 * MICRO;
		pub const COIN: Balance = 1_000 * MILLI;

		pub const fn deposit(items: u32, bytes: u32) -> Balance {
			(items as Balance) * 20 * COIN + (bytes as Balance) * 100 * MICRO
		}
	}
	pub use currency::*;

	pub mod time {
		// --- darwinia-network ---
		use darwinia_collator_primitives::*;

		pub const MILLISECS_PER_BLOCK: Moment = 12000;
		pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

		// Time is measured by number of blocks.
		pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
		pub const HOURS: BlockNumber = 60 * MINUTES;
		pub const DAYS: BlockNumber = 24 * HOURS;
	}
	pub use time::*;

	pub mod fee {
		// --- paritytech ---
		use frame_support::{
			traits::{Currency, Imbalance, OnUnbalanced},
			weights::{
				constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
				WeightToFeePolynomial,
			},
		};
		use pallet_transaction_payment::Multiplier;
		use sp_runtime::{FixedPointNumber, Perbill, Perquintill};
		// --- darwinia-network ---
		use crate::*;
		use darwinia_collator_primitives::*;

		frame_support::parameter_types! {
			/// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
			/// than this will decrease the weight and more will increase.
			pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
			/// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
			/// change the fees more rapidly.
			pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
			/// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
			/// that combined with `AdjustmentVariable`, we can recover from the minimum.
			/// See `multiplier_can_grow_from_zero`.
			pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
		}

		/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
		/// node's balance type.
		///
		/// This should typically create a mapping between the following ranges:
		///   - [0, MAXIMUM_BLOCK_WEIGHT]
		///   - [Balance::min, Balance::max]
		///
		/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
		///   - Setting it to `0` will essentially disable the weight fee.
		///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
		pub struct WeightToFee;
		impl WeightToFeePolynomial for WeightToFee {
			type Balance = Balance;
			fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
				// in `Crab Parachain`, extrinsic base weight (smallest non-zero weight) is mapped to 100 MILLI:
				let p = 100 * MILLI;
				let q = Balance::from(ExtrinsicBaseWeight::get());

				smallvec::smallvec![WeightToFeeCoefficient {
					degree: 1,
					negative: false,
					coeff_frac: Perbill::from_rational(p % q, q),
					coeff_integer: p / q,
				}]
			}
		}

		/// Logic for the author to get a portion of fees.
		pub struct ToStakingPot<R>(sp_std::marker::PhantomData<R>);
		impl<R> OnUnbalanced<NegativeImbalance<R>> for ToStakingPot<R>
		where
			R: pallet_balances::Config + pallet_collator_selection::Config,
			<R as frame_system::Config>::AccountId: From<AccountId>,
			<R as frame_system::Config>::AccountId: Into<AccountId>,
			<R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
		{
			fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
				let numeric_amount = amount.peek();
				let staking_pot = <pallet_collator_selection::Pallet<R>>::account_id();

				<pallet_balances::Pallet<R>>::resolve_creating(&staking_pot, amount);
				<frame_system::Pallet<R>>::deposit_event(pallet_balances::Event::Deposit {
					who: staking_pot,
					amount: numeric_amount,
				});
			}
		}

		pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);
		impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
		where
			R: pallet_balances::Config + pallet_collator_selection::Config,
			<R as frame_system::Config>::AccountId: From<AccountId>,
			<R as frame_system::Config>::AccountId: Into<AccountId>,
			<R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
		{
			fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
				if let Some(mut fees) = fees_then_tips.next() {
					if let Some(tips) = fees_then_tips.next() {
						tips.merge_into(&mut fees);
					}
					<ToStakingPot<R> as OnUnbalanced<_>>::on_unbalanced(fees);
				}
			}
		}
	}
	pub use fee::*;
}
pub use constants::*;

pub mod pallets;
pub use pallets::*;

pub mod weights;

pub mod wasm {
	//! Make the WASM binary available.

	#[cfg(all(
		feature = "std",
		any(target_arch = "x86_64", target_arch = "x86", target_vendor = "apple")
	))]
	include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

	#[cfg(all(
		feature = "std",
		not(any(target_arch = "x86_64", target_arch = "x86", target_vendor = "apple"))
	))]
	pub const WASM_BINARY: &[u8] = include_bytes!("../wasm/crab_parachain_runtime.compact.wasm");
	#[cfg(all(
		feature = "std",
		not(any(target_arch = "x86_64", target_arch = "x86", target_vendor = "apple"))
	))]
	pub const WASM_BINARY_BLOATY: &[u8] = include_bytes!("../wasm/crab_parachain_runtime.wasm");

	#[cfg(feature = "std")]
	/// Wasm binary unwrapped. If built with `BUILD_DUMMY_WASM_BINARY`, the function panics.
	pub fn wasm_binary_unwrap() -> &'static [u8] {
		#[cfg(all(
			feature = "std",
			any(target_arch = "x86_64", target_arch = "x86", target_vendor = "apple")
		))]
		return WASM_BINARY.expect(
			"Development wasm binary is not available. This means the client is \
							built with `BUILD_DUMMY_WASM_BINARY` flag and it is only usable for \
							production chains. Please rebuild with the flag disabled.",
		);
		#[cfg(all(
			feature = "std",
			not(any(target_arch = "x86_64", target_arch = "x86", target_vendor = "apple"))
		))]
		return WASM_BINARY;
	}
}
pub use wasm::*;

pub use darwinia_collator_primitives::*;

// --- paritytech ---
use sp_core::OpaqueMetadata;
use sp_runtime::{
	create_runtime_str, generic,
	traits::Block as BlockT,
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, MultiAddress,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;
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
	RemoveCollectiveFlip,
>;

pub struct RemoveCollectiveFlip;
impl frame_support::traits::OnRuntimeUpgrade for RemoveCollectiveFlip {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		use frame_support::storage::migration;
		// Remove the storage value `RandomMaterial` from removed pallet `RandomnessCollectiveFlip`
		migration::remove_storage_prefix(b"RandomnessCollectiveFlip", b"RandomMaterial", b"");
		<Runtime as frame_system::Config>::DbWeight::get().writes(1)
	}
}

type Ring = Balances;

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("Crab Parachain"),
	impl_name: create_runtime_str!("Darwinia Crab Parachain"),
	authoring_version: 1,
	spec_version: 2,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

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
		PolkadotXcm: pallet_xcm::{Pallet, Call, Storage, Event<T>, Origin} = 13,
		CumulusXcm: cumulus_pallet_xcm::{Pallet, Event<T>, Origin} = 14,
		DmpQueue: cumulus_pallet_dmp_queue::{Pallet, Call, Storage, Event<T>} = 15,

		// Handy utilities.
		Utility: pallet_utility::{Pallet, Call, Event} = 16,
		Multisig: pallet_multisig::{Pallet, Call, Storage, Event<T>} = 17,
		Proxy: pallet_proxy::{Pallet, Call, Storage, Event<T>} = 18,
		Sudo: pallet_sudo::{Pallet, Call, Storage, Config<T>, Event<T>} = 19,
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
		fn collect_collation_info() -> cumulus_primitives_core::CollationInfo {
			ParachainSystem::collect_collation_info()
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
