// --- paritytech --
use bp_messages::LaneId;
use bp_runtime::ChainId;
use frame_support::PalletId;
use pallet_bridge_messages::Instance1 as WithPangolinMessages;
use sp_core::H160;
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
	CallParams, ChainName, Config, CreatePayload, LatestMessageNoncer,
};
use frame_support::RuntimeDebug;
use scale_info::TypeInfo;
use sp_runtime::{traits::AccountIdConversion, AccountId32};

use pallet_bridge_messages::{inbound_lane, outbound_lane};

pub struct ToPangoroMessageSender;
impl LatestMessageNoncer for ToPangoroMessageSender {
	fn outbound_latest_generated_nonce(lane_id: LaneId) -> u64 {
		outbound_lane::<Runtime, WithPangolinMessages>(lane_id)
			.data()
			.latest_generated_nonce
			.into()
	}
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct ToPangoroOutboundPayLoad;
impl CreatePayload<AccountId, AccountPublic, Signature> for ToPangoroOutboundPayLoad {
	type Payload = ToPangolinMessagePayload;

	fn create(
		origin: CallOrigin<AccountId, AccountPublic, Signature>,
		spec_version: u32,
		weight: u64,
		call_params: CallParams,
		dispatch_fee_payment: DispatchFeePayment,
	) -> Result<Self::Payload, &'static str> {
		let call = Self::encode_call(PANGOLIN_S2S_BACKING_PALLET_INDEX, call_params)?;
		Ok(ToPangolinMessagePayload {
			spec_version,
			weight,
			origin,
			call,
			dispatch_fee_payment,
		})
	}
}

fn into_h160(pallet_id: &PalletId) -> H160 {
	let account_id: AccountId32 = pallet_id.into_account();
	let bytes: &[u8] = account_id.as_ref();
	H160::from_slice(&bytes[0..20])
}

frame_support::parameter_types! {
	pub const ParachainIssuingPalletId: PalletId = PalletId(*b"da/paais");
	pub const PangolinChainId: ChainId = PANGOLIN_CHAIN_ID;
	pub const BridgePangolinLaneId: LaneId = PANGOLIN_PANGOLIN_PARACHAIN_LANE;
	pub BackingChainName: ChainName = (b"Pangoro").to_vec();
	//pub RingAddress: H160 = PalletId(*b"da/bring").into_h160();
	pub RingAddress: H160 = into_h160(&PalletId(*b"da/bring"));
}

impl Config for Runtime {
	type PalletId = ParachainIssuingPalletId;
	type Event = Event;
	type WeightInfo = ();
	type RingCurrency = Ring;
	type BridgedAccountIdConverter = AccountIdConverter;
	type BridgedChainId = PangolinChainId;
	type OutboundPayloadCreator = ToPangoroOutboundPayLoad;
	type BackingChainName = BackingChainName;
	type MessageLaneId = BridgePangolinLaneId;
	type RingAddress = RingAddress;
	type MessagesBridge = BridgePangolinMessages;
	type MessageNoncer = ToPangoroMessageSender;
}
