// --- paritytech ---
use pallet_utility::Config;
// --- darwinia-network ---
use crate::*;

impl Config for Runtime {
	type Event = Event;
	type Call = Call;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = ();
}
