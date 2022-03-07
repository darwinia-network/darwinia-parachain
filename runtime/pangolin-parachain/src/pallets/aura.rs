pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;

// --- paritytech ---
use pallet_aura::Config;
// --- darwinia-network ---
use crate::*;

frame_support::parameter_types! {
	pub const MaxAuthorities: u32 = 100_000;
}

impl Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = MaxAuthorities;
}
