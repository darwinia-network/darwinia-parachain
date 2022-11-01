// --- paritytech ---
use pallet_collator_selection::IdentityCollator;
use pallet_session::{Config, PeriodicSessions};
use sp_runtime::traits::OpaqueKeys;
// --- darwinia-network ---
use crate::*;

sp_runtime::impl_opaque_keys! {
	pub struct SessionKeys {
		pub aura: Aura,
	}
}

frame_support::parameter_types! {
	pub const Period: u32 = 6 * HOURS;
	pub const Offset: u32 = 0;
}

impl Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Keys = SessionKeys;
	type NextSessionRotation = PeriodicSessions<Period, Offset>;
	// Essentially just Aura, but lets be pedantic.
	type SessionHandler = <SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type SessionManager = CollatorSelection;
	type ShouldEndSession = PeriodicSessions<Period, Offset>;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	// we don't have stash and controller, thus we don't need the convert as well.
	type ValidatorIdOf = IdentityCollator;
	type WeightInfo = ();
}
