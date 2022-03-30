pub use pallet_bridge_dispatch::Instance1 as WithPangolinDispatch;

// --- paritytech ---
use bp_messages::{LaneId, MessageNonce};
use bp_pangolin::AccountIdConverter;
use pallet_bridge_dispatch::Config;
// --- darwinia-network ---
use crate::{pangolin_messages::FromPangolinEncodedCall, *};

impl Config<WithPangolinDispatch> for Runtime {
	type Event = Event;
	type BridgeMessageId = (LaneId, MessageNonce);
	type Call = Call;

	/// TODO
	type CallFilter = frame_support::traits::Everything;
	type EncodedCall = FromPangolinEncodedCall;
	type SourceChainAccountId = bp_pangolin::AccountId;
	type TargetChainAccountPublic = AccountPublic;
	type TargetChainSignature = Signature;
	type AccountIdConverter = AccountIdConverter;
}
