pub use pallet_bridge_dispatch::Instance1 as WithCrabDispatch;

// --- paritytech ---
use frame_support::traits::Everything;
// --- darwinia-network ---
use crate::*;
use bp_messages::{LaneId, MessageNonce};
use pallet_bridge_dispatch::Config;

impl Config<WithCrabDispatch> for Runtime {
	type AccountIdConverter = bp_crab_parachain::AccountIdConverter;
	type BridgeMessageId = (LaneId, MessageNonce);
	type Call = Call;
	type CallFilter = Everything;
	type EncodedCall = bm_crab::FromCrabEncodedCall;
	type Event = Event;
	type SourceChainAccountId = bp_crab::AccountId;
	type TargetChainAccountPublic = AccountPublic;
	type TargetChainSignature = Signature;
}