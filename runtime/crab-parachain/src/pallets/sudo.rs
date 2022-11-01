// --- paritytech ---
use pallet_sudo::Config;
// --- darwinia-network ---
use crate::*;

impl Config for Runtime {
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
}
