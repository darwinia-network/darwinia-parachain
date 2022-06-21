// --- paritytech --
use bp_messages::LaneId;
use bp_runtime::ChainId;
use frame_support::PalletId;
use pallet_bridge_messages::Instance1 as WithPangolinMessages;
// --- darwinia-network ---
use crate::*;
use bp_message_dispatch::CallOrigin;
use bp_pangolin::AccountIdConverter;
use bp_runtime::{messages::DispatchFeePayment, PANGOLIN_CHAIN_ID};
use bridges_message::pangolin::{
	ToPangolinMessagePayload, PANGOLIN_PANGOLIN_PARACHAIN_LANE, PANGOLIN_S2S_BACKING_PALLET_INDEX,
};
use codec::{Decode, Encode};
use dc_common_runtime::helixbridge::{
	CallParams, Config, CreatePayload, LatestMessageNoncer,
};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;

use pallet_bridge_messages::outbound_lane;

pub struct ToPangolinMessageSender;
impl LatestMessageNoncer for ToPangolinMessageSender {
	fn outbound_latest_generated_nonce(lane_id: LaneId) -> u64 {
		outbound_lane::<Runtime, WithPangolinMessages>(lane_id).data().latest_generated_nonce.into()
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ToPangolinOutboundPayLoad;
impl CreatePayload<AccountId, AccountPublic, Signature, Runtime> for ToPangolinOutboundPayLoad {
	type Payload = ToPangolinMessagePayload;

	fn create(
		origin: CallOrigin<AccountId, AccountPublic, Signature>,
		spec_version: u32,
		weight: u64,
		call_params: CallParams<Runtime>,
		dispatch_fee_payment: DispatchFeePayment,
	) -> Result<Self::Payload, &'static str> {
		let call = Self::encode_call(PANGOLIN_S2S_BACKING_PALLET_INDEX, call_params)?;
		Ok(ToPangolinMessagePayload { spec_version, weight, origin, call, dispatch_fee_payment })
	}
}

frame_support::parameter_types! {
	pub const BridgePangolinLaneId: LaneId = PANGOLIN_PANGOLIN_PARACHAIN_LANE;
	pub const DecimalMultiplier: u128 = 1_000_000_000u128;
	pub const ParachainIssuingPalletId: PalletId = PalletId(*b"da/paais");
	pub const PangolinChainId: ChainId = PANGOLIN_CHAIN_ID;
}

impl Config for Runtime {
	type BridgedAccountIdConverter = AccountIdConverter;
	type BridgedChainId = PangolinChainId;
	type DecimalMultiplier = DecimalMultiplier;
	type Event = Event;
	type MessageLaneId = BridgePangolinLaneId;
	type MessageNoncer = ToPangolinMessageSender;
	type MessagesBridge = BridgePangolinMessages;
	type OutboundPayloadCreator = ToPangolinOutboundPayLoad;
	type PalletId = ParachainIssuingPalletId;
	type RingCurrency = Ring;
	type WeightInfo = ();
}
