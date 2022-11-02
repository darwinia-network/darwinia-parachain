// --- crates.io ---
use codec::{Decode, Encode};
use scale_info::TypeInfo;
// --- paritytech ---
use frame_support::{log, weights::Weight, RuntimeDebug};
use sp_runtime::{FixedPointNumber, FixedU128};
use sp_std::ops::RangeInclusive;
// --- darwinia-network ---
use crate::*;
use bp_messages::{source_chain::*, target_chain::*, *};
use bp_runtime::*;
use bridge_runtime_common::{
	lanes::*,
	messages::{source::*, target::*, *},
};
use dp_common_runtime::FromThisChainMessageVerifier;

/// Message delivery proof for CrabParachain -> Crab messages.
pub type ToCrabMessagesDeliveryProof = FromBridgedChainMessagesDeliveryProof<bp_crab::Hash>;
/// Message proof for Crab -> CrabParachain  messages.
pub type FromCrabMessagesProof = FromBridgedChainMessagesProof<bp_crab::Hash>;

/// Message payload for CrabParachain -> Crab messages.
pub type ToCrabMessagePayload = FromThisChainMessagePayload;
/// Message payload for Crab -> CrabParachain messages.
pub type FromCrabMessagePayload = FromBridgedChainMessagePayload<RuntimeCall>;

/// Message verifier for CrabParachain -> Crab messages.
pub type ToCrabMessageVerifier<R> =
	FromThisChainMessageVerifier<WithCrabMessageBridge, R, WithCrabFeeMarket>;

/// Call-dispatch based message dispatch for Crab -> CrabParachain messages.
pub type FromCrabMessageDispatch = FromBridgedChainMessageDispatch<
	WithCrabMessageBridge,
	xcm_executor::XcmExecutor<crate::polkadot_xcm::XcmConfig>,
	crate::polkadot_xcm::XcmWeigher,
	WeightCredit,
>;

pub const INITIAL_CRAB_TO_CRAB_PARACHAIN_CONVERSION_RATE: FixedU128 =
	FixedU128::from_inner(FixedU128::DIV);
/// Weight of 2 XCM instructions is for simple `Trap(42)` program, coming through bridge
/// (it is prepended with `UniversalOrigin` instruction). It is used just for simplest manual
/// tests, confirming that we don't break encoding somewhere between.
pub const BASE_XCM_WEIGHT_TWICE: u64 = 2 * crate::xcm_config::BASE_XCM_WEIGHT;

frame_support::parameter_types! {
	/// CrabParachain to Crab conversion rate. Initially we trate both tokens as equal.
	pub storage CrabToCrabParachainConversionRate: FixedU128 = INITIAL_CRAB_TO_CRAB_PARACHAIN_CONVERSION_RATE;
	/// Weight credit for our test messages.
	///
	/// 2 XCM instructions is for simple `Trap(42)` program, coming through bridge
	/// (it is prepended with `UniversalOrigin` instruction).
	pub const WeightCredit: Weight = Weight::from_ref_time(BASE_XCM_WEIGHT_TWICE);
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum CrabParachainToCrabParameter {
	/// The conversion formula we use is: `CrabTokens = CrabParachainTokens *
	/// conversion_rate`.
	CrabToCrabParachainConversionRate(FixedU128),
}
impl Parameter for CrabParachainToCrabParameter {
	fn save(&self) {
		match *self {
			CrabParachainToCrabParameter::CrabToCrabParachainConversionRate(
				ref conversion_rate,
			) => CrabToCrabParachainConversionRate::set(conversion_rate),
		}
	}
}

/// Crab <-> CrabParachain message bridge.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct WithCrabMessageBridge;
impl MessageBridge for WithCrabMessageBridge {
	type BridgedChain = Crab;
	type ThisChain = CrabParachain;

	const BRIDGED_CHAIN_ID: ChainId = CRAB_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bridge_runtime_common::WITH_CRAB_PARACHAIN_MESSAGES_PALLET_NAME;
	const RELAYER_FEE_PERCENT: u32 = 10;
	const THIS_CHAIN_ID: ChainId = CRAB_PARACHAIN_CHAIN_ID;
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct CrabParachain;
impl ChainWithMessages for CrabParachain {
	type AccountId = bp_crab_parachain::AccountId;
	type Balance = bp_crab_parachain::Balance;
	type Hash = bp_crab_parachain::Hash;
	type Signature = bp_crab_parachain::Signature;
	type Signer = bp_crab_parachain::AccountPublic;
}
impl ThisChainWithMessages for CrabParachain {
	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;

	fn is_message_accepted(send_origin: &Self::RuntimeOrigin, lane: &LaneId) -> bool {
		let here_location =
			xcm::v3::MultiLocation::from(crate::xcm_config::UniversalLocation::get());
		match send_origin.caller {
			OriginCaller::PolkadotXcm(pallet_xcm::Origin::Xcm(ref location))
				if *location == here_location =>
			{
				log::trace!(target: "runtime::bridge", "Verifying message sent using XCM pallet");
			},
			_ => {
				// keep in mind that in this case all messages are free (in term of fees)
				// => it's just to keep testing bridge on our test deployments until we'll have a
				// better option
				log::trace!(target: "runtime::bridge", "Verifying message sent using messages pallet");
			},
		}

		*lane == CRAB_CRAB_PARACHAIN_LANE || *lane == [0, 0, 0, 1]
	}

	fn maximal_pending_messages_at_outbound_lane() -> MessageNonce {
		MessageNonce::MAX
	}
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct Crab;
impl ChainWithMessages for Crab {
	type AccountId = bp_crab::AccountId;
	type Balance = bp_crab::Balance;
	type Hash = bp_crab::Hash;
	type Signature = bp_crab::Signature;
	type Signer = bp_crab::AccountPublic;
}
impl BridgedChainWithMessages for Crab {
	fn maximal_extrinsic_size() -> u32 {
		bp_crab::Crab::max_extrinsic_size()
	}

	fn verify_dispatch_weight(_message_payload: &[u8]) -> bool {
		true
	}
}
impl TargetHeaderChain<ToCrabMessagePayload, <Self as ChainWithMessages>::AccountId> for Crab {
	type Error = &'static str;
	type MessagesDeliveryProof = ToCrabMessagesDeliveryProof;

	fn verify_message(payload: &ToCrabMessagePayload) -> Result<(), Self::Error> {
		source::verify_chain_message::<WithCrabMessageBridge>(payload)
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<(LaneId, InboundLaneData<bp_crab::AccountId>), Self::Error> {
		source::verify_messages_delivery_proof::<WithCrabMessageBridge, Runtime, WithCrabGrandpa>(
			proof,
		)
	}
}
impl SourceHeaderChain<<Self as ChainWithMessages>::Balance> for Crab {
	type Error = &'static str;
	type MessagesProof = FromCrabMessagesProof;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message<<Self as ChainWithMessages>::Balance>>, Self::Error> {
		target::verify_messages_proof::<WithCrabMessageBridge, Runtime, WithCrabGrandpa>(
			proof,
			messages_count,
		)
	}
}

/// With-crab bridge
pub struct ToCrabBridge;

impl XcmBridge for ToCrabBridge {
	type MessageBridge = WithCrabMessageBridge;
	type MessageSender = pallet_bridge_messages::Pallet<Runtime, WithCrabMessagesInstance>;

	fn universal_location() -> InteriorMultiLocation {
		UniversalLocation::get()
	}

	fn verify_destination(dest: &MultiLocation) -> bool {
		// matches!(*dest, MultiLocation { parents: 1, interior: X2(GlobalConsensus(r),
		// Parachain(RIALTO_PARACHAIN_ID)) } if r == RialtoNetwork::get())
		unimplemented!("TODO")
	}

	fn build_destination() -> MultiLocation {
		let dest: InteriorMultiLocation = CrabNetwork::get().into();
		let here = UniversalLocation::get();
		dest.relative_to(&here)
	}

	fn xcm_lane() -> LaneId {
		CRAB_CRAB_PARACHAIN_LANE
	}
}
