pub use pallet_bridge_dispatch::Instance1 as WithPangolinDispatch;

// --- darwinia-network ---
use crate::{bridge_messages::pangolin::FromPangolinEncodedCall, *};
use bp_messages::{LaneId, MessageNonce};
use bp_pangolin::AccountIdConverter;
use pallet_bridge_dispatch::Config;

impl Config<WithPangolinDispatch> for Runtime {
	type Event = Event;
	type BridgeMessageId = (LaneId, MessageNonce);
	type Call = Call;

	// TODO
	type CallFilter = frame_support::traits::Everything;
	type EncodedCall = FromPangolinEncodedCall;
	type SourceChainAccountId = bp_pangolin::AccountId;
	type TargetChainAccountPublic = AccountPublic;
	type TargetChainSignature = Signature;
	type AccountIdConverter = AccountIdConverter;
}
