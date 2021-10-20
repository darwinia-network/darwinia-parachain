// --- paritytech ---
use cumulus_pallet_xcm::Config;
use xcm_executor::XcmExecutor;
// --- darwinia-network ---
use crate::*;

impl Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}
