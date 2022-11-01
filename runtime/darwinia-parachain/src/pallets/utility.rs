// --- paritytech ---
use pallet_utility::Config;
// --- darwinia-network ---
use crate::*;

impl Config for Runtime {
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}
