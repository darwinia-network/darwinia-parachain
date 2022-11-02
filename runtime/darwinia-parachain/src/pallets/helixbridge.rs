// --- crates.io ---
use codec::{Decode, Encode};
use scale_info::TypeInfo;
// --- paritytech --
use frame_support::{PalletId, RuntimeDebug};
// --- darwinia-network ---
use crate::*;
use bp_message_dispatch::CallOrigin;
use bp_messages::{LaneId, MessageNonce};
use bp_runtime::{messages::DispatchFeePayment, ChainId, DARWINIA_CHAIN_ID};
use bridge_runtime_common::lanes::DARWINIA_DARWINIA_PARACHAIN_LANE;
use bridges_message::darwinia::ToDarwiniaMessagePayload;
use dp_common_runtime::helixbridge::{CallParams, Config, CreatePayload, LatestMessageNoncer};
use pallet_bridge_messages::{outbound_lane, Instance1 as WithDarwiniaMessages};

/// The s2s backing pallet index in the darwinia chain runtime.
const DARWINIA_S2S_BACKING_PALLET_INDEX: u8 = 57;

pub struct ToDarwiniaMessageSender;
impl LatestMessageNoncer for ToDarwiniaMessageSender {
	fn outbound_latest_generated_nonce(lane_id: LaneId) -> MessageNonce {
		outbound_lane::<Runtime, WithDarwiniaMessages>(lane_id).data().latest_generated_nonce
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ToDarwiniaOutboundPayLoad;
impl CreatePayload<AccountId, AccountPublic, Signature, Runtime> for ToDarwiniaOutboundPayLoad {
	type Payload = ToDarwiniaMessagePayload;

	fn create(
		origin: CallOrigin<AccountId, AccountPublic, Signature>,
		spec_version: u32,
		weight: u64,
		call_params: CallParams<Runtime>,
		dispatch_fee_payment: DispatchFeePayment,
	) -> Result<Self::Payload, &'static str> {
		let call = Self::encode_call(DARWINIA_S2S_BACKING_PALLET_INDEX, call_params)?;
		Ok(ToDarwiniaMessagePayload { spec_version, weight, origin, call, dispatch_fee_payment })
	}
}

frame_support::parameter_types! {
	pub const BridgeDarwiniaLaneId: LaneId = DARWINIA_DARWINIA_PARACHAIN_LANE;
	pub const DecimalMultiplier: u128 = 1_000_000_000u128;
	pub const ParachainIssuingPalletId: PalletId = PalletId(*b"da/paais");
	pub const DarwiniaChainId: ChainId = DARWINIA_CHAIN_ID;
}

impl Config for Runtime {
	type BridgedAccountIdConverter = bp_darwinia::AccountIdConverter;
	type BridgedChainId = DarwiniaChainId;
	type DecimalMultiplier = DecimalMultiplier;
	type Event = Event;
	type MessageLaneId = BridgeDarwiniaLaneId;
	type MessageNoncer = ToDarwiniaMessageSender;
	type MessagesBridge = BridgeDarwiniaMessages;
	type OutboundPayloadCreator = ToDarwiniaOutboundPayLoad;
	type PalletId = ParachainIssuingPalletId;
	type RingCurrency = Ring;
	type WeightInfo = ();
}
