pub use pallet_bridge_dispatch::Instance1 as WithCrabDispatch;

// --- darwinia-network ---
use crate::*;
use bp_message_dispatch::{Everything, IntoDispatchOrigin as IntoDispatchOriginT};
use bp_messages::{LaneId, MessageNonce};
use pallet_bridge_dispatch::Config;

pub struct IntoDispatchOrigin;
impl IntoDispatchOriginT<bp_crab_parachain::AccountId, Call, Origin> for IntoDispatchOrigin {
	fn into_dispatch_origin(id: &bp_crab_parachain::AccountId, _: &Call) -> Origin {
		frame_system::RawOrigin::Signed(id.clone()).into()
	}
}

impl Config<WithCrabDispatch> for Runtime {
	type AccountIdConverter = bp_crab_parachain::AccountIdConverter;
	type BridgeMessageId = (LaneId, MessageNonce);
	type Call = Call;
	type CallValidator = Everything;
	type EncodedCall = bm_crab::FromCrabEncodedCall;
	type Event = Event;
	type IntoDispatchOrigin = IntoDispatchOrigin;
	type SourceChainAccountId = bp_crab::AccountId;
	type TargetChainAccountPublic = bp_crab_parachain::AccountPublic;
	type TargetChainSignature = bp_crab_parachain::Signature;
}
