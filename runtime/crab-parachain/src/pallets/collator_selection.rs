// --- paritytech ---
use frame_support::{traits::EnsureOneOf, PalletId};
use frame_system::EnsureRoot;
use pallet_collator_selection::Config;
use pallet_xcm::{EnsureXcm, IsMajorityOfBody};
use xcm::latest::BodyId;
// --- darwinia-network ---
use crate::*;

frame_support::parameter_types! {
	pub const ExecutiveBody: BodyId = BodyId::Executive;
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const MaxCandidates: u32 = 1000;
	pub const MinCandidates: u32 = 5;
	pub const MaxInvulnerables: u32 = 100;
}

/// We allow root and the Relay Chain council to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin =
	EnsureOneOf<EnsureRoot<AccountId>, EnsureXcm<IsMajorityOfBody<KsmLocation, ExecutiveBody>>>;

impl Config for Runtime {
	type Event = Event;
	type Currency = Ring;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type PotId = PotId;
	type MaxCandidates = MaxCandidates;
	type MinCandidates = MinCandidates;
	type MaxInvulnerables = MaxInvulnerables;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_collator_selection::IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = weights::pallet_collator_selection::WeightInfo<Runtime>;
}
