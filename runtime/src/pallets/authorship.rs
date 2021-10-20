// --- paritytech ---
use pallet_authorship::Config;
use pallet_session::FindAccountFromAuthorIndex;
// --- darwinia-network ---
use crate::*;

frame_support::parameter_types! {
	pub const UncleGenerations: u32 = 0;
}

impl Config for Runtime {
	type FindAuthor = FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = (CollatorSelection,);
}
