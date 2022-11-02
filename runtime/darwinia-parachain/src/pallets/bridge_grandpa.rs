pub use pallet_bridge_grandpa::Instance1 as WithDarwiniaGrandpa;

// --- darwinia-network ---
use crate::*;
use pallet_bridge_grandpa::Config;

frame_support::parameter_types! {
	// This is a pretty unscientific cap.
	//
	// Note that once this is hit the pallet will essentially throttle incoming requests down to one
	// call per block.
	pub const MaxRequests: u32 = 50;
	pub const DarwiniaHeadersToKeep: u32 = 500;
}

impl Config<WithDarwiniaGrandpa> for Runtime {
	type BridgedChain = bp_darwinia::Darwinia;
	type HeadersToKeep = DarwiniaHeadersToKeep;
	type MaxRequests = MaxRequests;
	type WeightInfo = ();
}
