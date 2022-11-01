// --- paritytech ---
use cumulus_pallet_dmp_queue::Config;
use frame_system::EnsureRoot;
use xcm_executor::XcmExecutor;
// --- darwinia-network ---
use crate::*;

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}
