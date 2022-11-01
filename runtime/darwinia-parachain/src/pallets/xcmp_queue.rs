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
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type PriceForSiblingDelivery = ();
	type RuntimeEvent = RuntimeEvent;
	type VersionWrapper = PolkadotXcm;
	type WeightInfo = ();
	type XcmExecutor = XcmExecutor<XcmConfig>;
}
