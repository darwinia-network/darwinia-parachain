// --- parity ---
use cumulus_pallet_xcmp_queue::Config;
// --- darwinia ---
use crate::*;

impl Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
}
