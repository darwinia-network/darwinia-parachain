// --- paritytech ---
use cumulus_pallet_parachain_system::{Config, RelayNumberStrictlyIncreases};
use frame_support::weights::Weight;
use parachain_info::Pallet as ParachainInfoPallet;
// --- darwinia-network ---
use crate::*;

frame_support::parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

impl Config for Runtime {
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
	type DmpMessageHandler = DmpQueue;
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type OutboundXcmpMessageSource = XcmpQueue;
	type ReservedDmpWeight = ReservedDmpWeight;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type SelfParaId = ParachainInfoPallet<Runtime>;
	type XcmpMessageHandler = XcmpQueue;
}
