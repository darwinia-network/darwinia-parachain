// --- crates.io ---
use codec::{Decode, Encode};
use scale_info::TypeInfo;
// --- paritytech ---
use frame_support::PalletId;
// --- darwinia-network ---
use crate::*;
use bp_message_dispatch::CallOrigin;
use bp_messages::{LaneId, MessageNonce};
use bp_runtime::{messages::DispatchFeePayment, ChainId, PANGOLIN_CHAIN_ID};
use bridge_runtime_common::lanes::PANGOLIN_PANGOLIN_PARACHAIN_LANE;
use bridges_message::pangolin::{ToPangolinMessagePayload, ETHEREUM_PALLET_INDEX};
use dp_common_runtime::helixbridge::{
	evm::ConcatConverter, CallParams, Config, CreatePayload, LatestMessageNoncer,
};
use frame_support::RuntimeDebug;
use pallet_bridge_messages::Instance1 as WithPangolinMessages;

pub struct ToPangolinMessageSender;
impl LatestMessageNoncer for ToPangolinMessageSender {
	fn outbound_latest_generated_nonce(lane_id: LaneId) -> MessageNonce {
		pallet_bridge_messages::OutboundLanes::<Runtime, WithPangolinMessages>::get(&lane_id)
			.latest_generated_nonce
	}

	fn inbound_latest_received_nonce(lane_id: LaneId) -> MessageNonce {
		pallet_bridge_messages::InboundLanes::<Runtime, WithPangolinMessages>::get(&lane_id)
			.last_delivered_nonce()
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ToPangolinOutboundPayLoad;
impl CreatePayload<AccountId, AccountPublic, Signature> for ToPangolinOutboundPayLoad {
	type Payload = ToPangolinMessagePayload;

	fn create(
		origin: CallOrigin<AccountId, AccountPublic, Signature>,
		spec_version: u32,
		weight: u64,
		call_params: CallParams,
		dispatch_fee_payment: DispatchFeePayment,
	) -> Result<Self::Payload, &'static str> {
		let call = Self::encode_call(ETHEREUM_PALLET_INDEX, call_params)?;
		Ok(ToPangolinMessagePayload { spec_version, weight, origin, call, dispatch_fee_payment })
	}
}

frame_support::parameter_types! {
	pub const BridgePangolinLaneId: LaneId = PANGOLIN_PANGOLIN_PARACHAIN_LANE;
	pub const ParachainIssuingPalletId: PalletId = PalletId(*b"da/paais");
	pub const PangolinChainId: ChainId = PANGOLIN_CHAIN_ID;
	pub const PangolinSmartChainId: u64 = 43;
	pub const MaxNonceReserves: u32 = 1024;
}

impl Config for Runtime {
	type BridgedAccountIdConverter = bp_pangolin::AccountIdConverter;
	type BridgedChainId = PangolinChainId;
	type BridgedSmartChainId = PangolinSmartChainId;
	type Event = Event;
	type IntoEthereumAccount = ConcatConverter<Self::AccountId>;
	type MaxReserves = MaxNonceReserves;
	type MessageLaneId = BridgePangolinLaneId;
	type MessageNoncer = ToPangolinMessageSender;
	type MessagesBridge = BridgePangolinMessages;
	type OutboundPayloadCreator = ToPangolinOutboundPayLoad;
	type PalletId = ParachainIssuingPalletId;
	type RingCurrency = Ring;
	type WeightInfo = ();
}
