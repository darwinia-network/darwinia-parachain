// --- paritytech ---
use pallet_utility::Config;
// --- darwinia-network ---
use crate::{weights::pallet_utility::WeightInfo, *};

impl Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = WeightInfo<Runtime>;
}
