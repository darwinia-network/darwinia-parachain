// --- parity ---
use frame_system::Config as SystemConfig;
use pallet_collator_selection::IdentityCollator;
use pallet_session::{Config, PeriodicSessions};
use sp_runtime::{traits::OpaqueKeys, Perbill};
// --- darwinia ---
use crate::{weights::pallet_session::WeightInfo, *};

sp_runtime::impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

frame_support::parameter_types! {
	pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}

impl Config for Runtime {
	type Event = Event;
	type ValidatorId = <Self as SystemConfig>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = IdentityCollator;
	type ShouldEndSession = PeriodicSessions<Period, Offset>;
	type NextSessionRotation = PeriodicSessions<Period, Offset>;
	type SessionManager = CollatorSelection;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = SessionKeys;
	type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
	type WeightInfo = WeightInfo<Runtime>;
}
