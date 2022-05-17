// --- paritytech ---
use cumulus_pallet_xcmp_queue::Config;
use frame_system::EnsureRoot;
use xcm_executor::XcmExecutor;
// --- darwinia-network ---
use crate::*;

impl Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = ();
}
