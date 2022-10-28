// --- crates.io ---
use codec::{Decode, Encode};
use scale_info::TypeInfo;
// --- paritytech --
use frame_support::{PalletId, RuntimeDebug};
// --- darwinia-network ---
use crate::*;
use bp_message_dispatch::CallOrigin;
use bp_messages::{LaneId, MessageNonce};
use bp_runtime::{messages::DispatchFeePayment, ChainId, CRAB_CHAIN_ID};
use bridge_runtime_common::lanes::CRAB_CRAB_PARACHAIN_LANE;
use bridges_message::crab::ToCrabMessagePayload;
use dp_common_runtime::helixbridge::{
	evm::ConcatConverter, CallParams, Config, CreatePayload, LatestMessageNoncer,
};
use pallet_bridge_messages::Instance1 as WithCrabMessages;

/// The s2s backing pallet index in the crab chain runtime.
const CRAB_ETHEREUM_PALLET_INDEX: u8 = 40;

pub struct ToCrabMessageSender;
impl LatestMessageNoncer for ToCrabMessageSender {
	fn outbound_latest_generated_nonce(lane_id: LaneId) -> MessageNonce {
		pallet_bridge_messages::OutboundLanes::<Runtime, WithCrabMessages>::get(&lane_id)
			.latest_generated_nonce
	}

	fn inbound_latest_received_nonce(lane_id: LaneId) -> MessageNonce {
		pallet_bridge_messages::InboundLanes::<Runtime, WithCrabMessages>::get(&lane_id)
			.last_delivered_nonce()
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ToCrabOutboundPayLoad;
impl CreatePayload<AccountId, AccountPublic, Signature> for ToCrabOutboundPayLoad {
	type Payload = ToCrabMessagePayload;

	fn create(
		origin: CallOrigin<AccountId, AccountPublic, Signature>,
		spec_version: u32,
		weight: u64,
		call_params: CallParams,
		dispatch_fee_payment: DispatchFeePayment,
	) -> Result<Self::Payload, &'static str> {
		let call = Self::encode_call(CRAB_ETHEREUM_PALLET_INDEX, call_params)?;
		Ok(ToCrabMessagePayload { spec_version, weight, origin, call, dispatch_fee_payment })
	}
}

frame_support::parameter_types! {
	pub const BridgeCrabLaneId: LaneId = CRAB_CRAB_PARACHAIN_LANE;
	pub const ParachainIssuingPalletId: PalletId = PalletId(*b"da/paais");
	pub const CrabChainId: ChainId = CRAB_CHAIN_ID;
	pub const CrabSmartChainId: u64 = 44;
	pub const MaxNonceReserves: u32 = 1024;
}

impl Config for Runtime {
	type BridgedAccountIdConverter = bp_crab::AccountIdConverter;
	type BridgedChainId = CrabChainId;
	type BridgedSmartChainId = CrabSmartChainId;
	type Event = Event;
	type IntoEthereumAccount = ConcatConverter<Self::AccountId>;
	type MaxReserves = MaxNonceReserves;
	type MessageLaneId = BridgeCrabLaneId;
	type MessageNoncer = ToCrabMessageSender;
	type MessagesBridge = BridgeCrabMessages;
	type OutboundPayloadCreator = ToCrabOutboundPayLoad;
	type PalletId = ParachainIssuingPalletId;
	type RingCurrency = Ring;
	type WeightInfo = ();
}
