// --- parity ---
use cumulus_pallet_xcmp_queue::Config;
use xcm_executor::XcmExecutor;
// --- darwinia ---
use crate::*;

impl Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = ();
}
