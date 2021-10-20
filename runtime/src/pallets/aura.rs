pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;

// --- parity ---
use pallet_aura::Config;
// --- darwinia ---
use crate::*;

impl Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
}
