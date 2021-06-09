// --- parity ---
use pallet_xcm::Config;
// --- darwinia ---
use crate::*;

impl Config for Runtime {
	type Event = Event;
	type SendXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}
