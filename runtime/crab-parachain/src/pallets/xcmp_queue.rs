// --- paritytech ---
use cumulus_pallet_xcmp_queue::Config;
use frame_system::EnsureRoot;
use xcm_executor::XcmExecutor;
// --- darwinia-network ---
use crate::*;

impl Config for Runtime {
	type ChannelInfo = ParachainSystem;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type Event = Event;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type VersionWrapper = PolkadotXcm;
	// TODO: Update weight
	type WeightInfo = ();
	type XcmExecutor = XcmExecutor<XcmConfig>;
}
