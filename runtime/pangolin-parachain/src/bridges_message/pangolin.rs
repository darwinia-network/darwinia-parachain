// --- crates.io ---
use codec::{Decode, Encode};
use scale_info::TypeInfo;
// --- paritytech ---
use frame_support::{log, weights::Weight, RuntimeDebug};
use sp_runtime::{FixedPointNumber, FixedU128};
use sp_std::ops::RangeInclusive;
use xcm::latest::prelude::*;
// --- darwinia-network ---
use crate::*;
use bp_messages::{source_chain::*, target_chain::*, *};
use bp_runtime::*;
use bridge_runtime_common::{
	lanes::PANGOLIN_PANGOLIN_PARACHAIN_LANE,
	messages::{source::*, target::*, *},
};
use dp_common_runtime::FromThisChainMessageVerifier;

/// Message delivery proof for PangolinParachain -> Pangolin messages.
pub type ToPangolinMessagesDeliveryProof = FromBridgedChainMessagesDeliveryProof<bp_pangolin::Hash>;
/// Message proof for Pangolin -> PangolinParachain  messages.
pub type FromPangolinMessagesProof = FromBridgedChainMessagesProof<bp_pangolin::Hash>;

/// Message payload for PangolinParachain -> Pangolin messages.
pub type ToPangolinMessagePayload = FromThisChainMessagePayload;
/// Message payload for Pangolin -> PangolinParachain messages.
pub type FromPangolinMessagePayload = FromBridgedChainMessagePayload<RuntimeCall>;

/// Message verifier for PangolinParachain -> Pangolin messages.
pub type ToPangolinMessageVerifier<R> =
	FromThisChainMessageVerifier<WithPangolinMessageBridge, R, WithPangolinFeeMarket>;

/// Call-dispatch based message dispatch for Pangolin -> PangolinParachain messages.
pub type FromPangolinMessageDispatch = FromBridgedChainMessageDispatch<
	WithPangolinMessageBridge,
	xcm_executor::XcmExecutor<XcmConfig>,
	XcmWeigher,
	WeightCredit,
>;

pub const INITIAL_PANGOLIN_TO_PANGOLIN_PARACHAIN_CONVERSION_RATE: FixedU128 =
	FixedU128::from_inner(FixedU128::DIV);
/// Weight of 2 XCM instructions is for simple `Trap(42)` program, coming through bridge
/// (it is prepended with `UniversalOrigin` instruction). It is used just for simplest manual
/// tests, confirming that we don't break encoding somewhere between.
pub const BASE_XCM_WEIGHT_TWICE: u64 = 2 * BASE_XCM_WEIGHT;

frame_support::parameter_types! {
	/// PangolinParachain to Pangolin conversion rate. Initially we trate both tokens as equal.
	pub storage PangolinToPangolinParachainConversionRate: FixedU128 = INITIAL_PANGOLIN_TO_PANGOLIN_PARACHAIN_CONVERSION_RATE;
	/// Weight credit for our test messages.
	///
	/// 2 XCM instructions is for simple `Trap(42)` program, coming through bridge
	/// (it is prepended with `UniversalOrigin` instruction).
	pub const WeightCredit: Weight = Weight::from_ref_time(BASE_XCM_WEIGHT_TWICE);
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum PangolinParachainToPangolinParameter {
	/// The conversion formula we use is: `PangolinTokens = PangolinParachainTokens *
	/// conversion_rate`.
	PangolinToPangolinParachainConversionRate(FixedU128),
}
impl Parameter for PangolinParachainToPangolinParameter {
	fn save(&self) {
		match *self {
			PangolinParachainToPangolinParameter::PangolinToPangolinParachainConversionRate(
				ref conversion_rate,
			) => PangolinToPangolinParachainConversionRate::set(conversion_rate),
		}
	}
}

/// Pangolin <-> PangolinParachain message bridge.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct WithPangolinMessageBridge;
impl MessageBridge for WithPangolinMessageBridge {
	type BridgedChain = Pangolin;
	type ThisChain = PangolinParachain;

	const BRIDGED_CHAIN_ID: ChainId = PANGOLIN_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bridge_runtime_common::pallets::WITH_PANGOLIN_PARACHAIN_MESSAGES_PALLET_NAME;
	const RELAYER_FEE_PERCENT: u32 = 10;
	const THIS_CHAIN_ID: ChainId = PANGOLIN_PARACHAIN_CHAIN_ID;
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct PangolinParachain;
impl ChainWithMessages for PangolinParachain {
	type AccountId = bp_pangolin_parachain::AccountId;
	type Balance = bp_pangolin_parachain::Balance;
	type Hash = bp_pangolin_parachain::Hash;
	type Signature = bp_pangolin_parachain::Signature;
	type Signer = bp_pangolin_parachain::AccountPublic;
}
impl ThisChainWithMessages for PangolinParachain {
	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;

	fn is_message_accepted(send_origin: &Self::RuntimeOrigin, lane: &LaneId) -> bool {
		let here_location = xcm::latest::MultiLocation::from(UniversalLocation::get());
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

		*lane == PANGOLIN_PANGOLIN_PARACHAIN_LANE || *lane == [0, 0, 0, 1]
	}

	fn maximal_pending_messages_at_outbound_lane() -> MessageNonce {
		MessageNonce::MAX
	}
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct Pangolin;
impl ChainWithMessages for Pangolin {
	type AccountId = bp_pangolin::AccountId;
	type Balance = bp_pangolin::Balance;
	type Hash = bp_pangolin::Hash;
	type Signature = bp_pangolin::Signature;
	type Signer = bp_pangolin::AccountPublic;
}
impl BridgedChainWithMessages for Pangolin {
	fn maximal_extrinsic_size() -> u32 {
		bp_pangolin::DarwiniaLike::max_extrinsic_size()
	}

	fn verify_dispatch_weight(_message_payload: &[u8]) -> bool {
		true
	}
}
impl TargetHeaderChain<ToPangolinMessagePayload, <Self as ChainWithMessages>::AccountId>
	for Pangolin
{
	type Error = &'static str;
	type MessagesDeliveryProof = ToPangolinMessagesDeliveryProof;

	fn verify_message(payload: &ToPangolinMessagePayload) -> Result<(), Self::Error> {
		source::verify_chain_message::<WithPangolinMessageBridge>(payload)
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<(LaneId, InboundLaneData<bp_pangolin::AccountId>), Self::Error> {
		source::verify_messages_delivery_proof::<
			WithPangolinMessageBridge,
			Runtime,
			WithPangolinGrandpa,
		>(proof)
	}
}
impl SourceHeaderChain<<Self as ChainWithMessages>::Balance> for Pangolin {
	type Error = &'static str;
	type MessagesProof = FromPangolinMessagesProof;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message<<Self as ChainWithMessages>::Balance>>, Self::Error> {
		target::verify_messages_proof::<WithPangolinMessageBridge, Runtime, WithPangolinGrandpa>(
			proof,
			messages_count,
		)
	}
}

/// The s2s backing pallet index in the pangoro chain runtime.
pub const PANGOLIN_S2S_BACKING_PALLET_INDEX: u8 = 65;

/// With-pangolin bridge
pub struct ToPangolinBridge;

impl XcmBridge for ToPangolinBridge {
	type MessageBridge = WithPangolinMessageBridge;
	type MessageSender = pallet_bridge_messages::Pallet<Runtime, WithPangolinMessages>;

	fn universal_location() -> InteriorMultiLocation {
		UniversalLocation::get()
	}

	fn verify_destination(dest: &MultiLocation) -> bool {
		// matches!(*dest, MultiLocation { parents: 1, interior: X2(GlobalConsensus(r),
		// Parachain(RIALTO_PARACHAIN_ID)) } if r == RialtoNetwork::get())
		unimplemented!("TODO")
	}

	fn build_destination() -> MultiLocation {
		let dest: InteriorMultiLocation = PangolinNetwork::get().into();
		let here = UniversalLocation::get();
		dest.relative_to(&here)
	}

	fn xcm_lane() -> LaneId {
		PANGOLIN_PANGOLIN_PARACHAIN_LANE
	}
}
