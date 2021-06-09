// --- substrate ---
use cumulus_pallet_parachain_system::Config;
use parachain_info::Pallet as ParachainInfoPallet;
// --- darwinia ---
use crate::*;

impl Config for Runtime {
	type Event = Event;
	type OnValidationData = ();
	type SelfParaId = ParachainInfoPallet<Runtime>;
	type DownwardMessageHandlers = ();
	type HrmpMessageHandlers = ();
}
