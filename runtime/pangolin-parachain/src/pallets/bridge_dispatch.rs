pub use pallet_bridge_dispatch::Instance1 as WithPangolinDispatch;

// --- darwinia-network ---
use crate::*;
use bp_message_dispatch::{Everything, IntoDispatchOrigin as IntoDispatchOriginT};
use bp_messages::{LaneId, MessageNonce};
use pallet_bridge_dispatch::Config;

pub struct IntoDispatchOrigin;
impl IntoDispatchOriginT<bp_pangolin_parachain::AccountId, Call, Origin> for IntoDispatchOrigin {
	fn into_dispatch_origin(id: &bp_pangolin_parachain::AccountId, _: &Call) -> Origin {
		frame_system::RawOrigin::Signed(id.clone()).into()
	}
}

impl Config<WithPangolinDispatch> for Runtime {
	type AccountIdConverter = bp_pangolin_parachain::AccountIdConverter;
	type BridgeMessageId = (LaneId, MessageNonce);
	type Call = Call;
	type CallValidator = Everything;
	type EncodedCall = bm_pangolin::FromPangolinEncodedCall;
	type Event = Event;
	type IntoDispatchOrigin = IntoDispatchOrigin;
	type SourceChainAccountId = bp_pangolin::AccountId;
	type TargetChainAccountPublic = AccountPublic;
	type TargetChainSignature = Signature;
}
