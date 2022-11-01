// --- paritytech ---
use pallet_utility::Config;
// --- darwinia-network ---
use crate::*;

impl Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}
