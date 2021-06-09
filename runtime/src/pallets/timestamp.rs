// --- substrate ---
use pallet_timestamp::Config;
// --- darwinia ---
use crate::*;

frame_support::parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}
impl Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = Moment;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}
