pub use pallet_bridge_dispatch::Instance1 as WithPangolinDispatch;

// --- paritytech ---
use frame_support::traits::Everything;
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
	type CallFilter = Everything;
	type EncodedCall = FromPangolinEncodedCall;
	type SourceChainAccountId = bp_pangolin::AccountId;
	type TargetChainAccountPublic = AccountPublic;
	type TargetChainSignature = Signature;
	type AccountIdConverter = AccountIdConverter;
}
