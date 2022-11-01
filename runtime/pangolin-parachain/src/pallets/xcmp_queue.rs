// --- paritytech ---
use cumulus_pallet_xcmp_queue::Config;
use frame_system::EnsureRoot;
use xcm_executor::XcmExecutor;
// --- darwinia-network ---
use crate::{weights::cumulus_pallet_xcmp_queue::WeightInfo, *};

impl Config for Runtime {
	type ChannelInfo = ParachainSystem;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type RuntimeEvent = RuntimeEvent;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type VersionWrapper = PolkadotXcm;
	type WeightInfo = WeightInfo<Runtime>;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}
