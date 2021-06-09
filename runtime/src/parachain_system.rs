// --- substrate ---
use cumulus_pallet_parachain_system::Config;
use parachain_info::Module as ParachainInfoModule;
// --- darwinia ---
use crate::*;

impl Config for Runtime {
	type Event = Event;
	type OnValidationData = ();
	type SelfParaId = ParachainInfoModule<Runtime>;
	type DownwardMessageHandlers = XcmHandler;
	type HrmpMessageHandlers = XcmHandler;
}
