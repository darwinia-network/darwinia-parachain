// --- parity ---
use cumulus_pallet_parachain_system::Config;
use cumulus_primitives_utility::UnqueuedDmpAsParent;
use frame_support::weights::Weight;
use parachain_info::Pallet as ParachainInfoPallet;
use xcm_executor::XcmExecutor;
// --- darwinia ---
use crate::*;

frame_support::parameter_types! {
	pub const ReservedXcmpWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 4;
	pub const MaxDownwardMessageWeight: Weight = MAXIMUM_BLOCK_WEIGHT / 10;
}
impl Config for Runtime {
	type Event = Event;
	type OnValidationData = ();
	type SelfParaId = ParachainInfoPallet<Runtime>;
	type DownwardMessageHandlers =
		UnqueuedDmpAsParent<MaxDownwardMessageWeight, XcmExecutor<XcmConfig>, Call>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
}
