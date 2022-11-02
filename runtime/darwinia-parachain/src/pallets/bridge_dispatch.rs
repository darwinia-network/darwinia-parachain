pub use pallet_bridge_dispatch::Instance1 as WithDarwiniaDispatch;

// --- darwinia-network ---
use crate::*;
use bp_message_dispatch::{Everything, IntoDispatchOrigin as IntoDispatchOriginT};
use bp_messages::{LaneId, MessageNonce};
use pallet_bridge_dispatch::Config;

pub struct IntoDispatchOrigin;
impl IntoDispatchOriginT<bp_darwinia_parachain::AccountId, Call, Origin> for IntoDispatchOrigin {
	fn into_dispatch_origin(id: &bp_darwinia_parachain::AccountId, _: &Call) -> Origin {
		frame_system::RawOrigin::Signed(id.clone()).into()
	}
}

impl Config<WithDarwiniaDispatch> for Runtime {
	type AccountIdConverter = bp_darwinia_parachain::AccountIdConverter;
	type BridgeMessageId = (LaneId, MessageNonce);
	type Call = Call;
	type CallValidator = Everything;
	type EncodedCall = bm_darwinia::FromDarwiniaEncodedCall;
	type Event = Event;
	type IntoDispatchOrigin = IntoDispatchOrigin;
	type SourceChainAccountId = bp_darwinia::AccountId;
	type TargetChainAccountPublic = bp_darwinia_parachain::AccountPublic;
	type TargetChainSignature = bp_darwinia_parachain::Signature;
}
