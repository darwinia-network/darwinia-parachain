// --- crates.io ---
use codec::{Decode, Encode};
use scale_info::TypeInfo;
// --- paritytech --
use bp_messages::LaneId;
use bp_runtime::ChainId;
use frame_support::{PalletId, RuntimeDebug};
use pallet_bridge_messages::Instance1 as WithCrabMessages;
// --- darwinia-network ---
use crate::*;
use bp_message_dispatch::CallOrigin;
use bp_runtime::{messages::DispatchFeePayment, CRAB_CHAIN_ID};
use bridge_runtime_common::lanes::CRAB_CRAB_PARACHAIN_LANE;
use bridges_message::crab::ToCrabMessagePayload;
use dp_common_runtime::helixbridge::{CallParams, Config, CreatePayload, LatestMessageNoncer};
use pallet_bridge_messages::outbound_lane;

/// The s2s backing pallet index in the crab chain runtime.
const CRAB_S2S_BACKING_PALLET_INDEX: u8 = 57;

pub struct ToCrabMessageSender;
impl LatestMessageNoncer for ToCrabMessageSender {
	fn outbound_latest_generated_nonce(lane_id: LaneId) -> u64 {
		outbound_lane::<Runtime, WithCrabMessages>(lane_id).data().latest_generated_nonce.into()
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ToCrabOutboundPayLoad;
impl CreatePayload<AccountId, AccountPublic, Signature, Runtime> for ToCrabOutboundPayLoad {
	type Payload = ToCrabMessagePayload;

	fn create(
		origin: CallOrigin<AccountId, AccountPublic, Signature>,
		spec_version: u32,
		weight: u64,
		call_params: CallParams<Runtime>,
		dispatch_fee_payment: DispatchFeePayment,
	) -> Result<Self::Payload, &'static str> {
		let call = Self::encode_call(CRAB_S2S_BACKING_PALLET_INDEX, call_params)?;
		Ok(ToCrabMessagePayload { spec_version, weight, origin, call, dispatch_fee_payment })
	}
}

frame_support::parameter_types! {
	pub const BridgeCrabLaneId: LaneId = CRAB_CRAB_PARACHAIN_LANE;
	pub const DecimalMultiplier: u128 = 1_000_000_000u128;
	pub const ParachainIssuingPalletId: PalletId = PalletId(*b"da/paais");
	pub const CrabChainId: ChainId = CRAB_CHAIN_ID;
}

impl Config for Runtime {
	type BridgedAccountIdConverter = bp_crab::AccountIdConverter;
	type BridgedChainId = CrabChainId;
	type DecimalMultiplier = DecimalMultiplier;
	type Event = Event;
	type MessageLaneId = BridgeCrabLaneId;
	type MessageNoncer = ToCrabMessageSender;
	type MessagesBridge = BridgeCrabMessages;
	type OutboundPayloadCreator = ToCrabOutboundPayLoad;
	type PalletId = ParachainIssuingPalletId;
	type RingCurrency = Ring;
	type WeightInfo = ();
}
