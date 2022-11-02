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
use dp_common_runtime::helixbridge::{
	evm::ConcatConverter, CallParams, Config, CreatePayload, LatestMessageNoncer,
};
use pallet_bridge_messages::Instance1 as WithDarwiniaMessages;

/// The ethereum pallet index in the darwinia chain runtime.
const DARWINIA_ETHEREUM_PALLET_INDEX: u8 = 48;

pub struct ToDarwiniaMessageSender;
impl LatestMessageNoncer for ToDarwiniaMessageSender {
	fn outbound_latest_generated_nonce(lane_id: LaneId) -> MessageNonce {
		pallet_bridge_messages::OutboundLanes::<Runtime, WithDarwiniaMessages>::get(&lane_id)
			.latest_generated_nonce
	}

	fn inbound_latest_received_nonce(lane_id: LaneId) -> MessageNonce {
		pallet_bridge_messages::InboundLanes::<Runtime, WithDarwiniaMessages>::get(&lane_id)
			.last_delivered_nonce()
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ToDarwiniaOutboundPayLoad;
impl CreatePayload<AccountId, AccountPublic, Signature> for ToDarwiniaOutboundPayLoad {
	type Payload = ToDarwiniaMessagePayload;

	fn create(
		origin: CallOrigin<AccountId, AccountPublic, Signature>,
		spec_version: u32,
		weight: u64,
		call_params: CallParams,
		dispatch_fee_payment: DispatchFeePayment,
	) -> Result<Self::Payload, &'static str> {
		let call = Self::encode_call(DARWINIA_ETHEREUM_PALLET_INDEX, call_params)?;
		Ok(ToDarwiniaMessagePayload { spec_version, weight, origin, call, dispatch_fee_payment })
	}
}

frame_support::parameter_types! {
	pub const BridgeDarwiniaLaneId: LaneId = DARWINIA_DARWINIA_PARACHAIN_LANE;
	pub const ParachainIssuingPalletId: PalletId = PalletId(*b"da/paais");
	pub const DarwiniaChainId: ChainId = DARWINIA_CHAIN_ID;
	pub const DarwiniaSmartChainId: u64 = 46;
	pub const MaxNonceReserves: u32 = 4096;
}

impl Config for Runtime {
	type BridgedAccountIdConverter = bp_darwinia::AccountIdConverter;
	type BridgedChainId = DarwiniaChainId;
	type BridgedSmartChainId = DarwiniaSmartChainId;
	type Event = Event;
	type IntoEthereumAccount = ConcatConverter<Self::AccountId>;
	type MaxReserves = MaxNonceReserves;
	type MessageLaneId = BridgeDarwiniaLaneId;
	type MessageNoncer = ToDarwiniaMessageSender;
	type MessagesBridge = BridgeDarwiniaMessages;
	type OutboundPayloadCreator = ToDarwiniaOutboundPayLoad;
	type PalletId = ParachainIssuingPalletId;
	type RingCurrency = Ring;
	type WeightInfo = ();
}
