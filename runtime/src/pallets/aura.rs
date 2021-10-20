pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;

// --- paritytech ---
use pallet_aura::Config;
// --- darwinia-network ---
use crate::*;

impl Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
}
