// --- paritytech ---
use frame_support::{traits::EnsureOneOf, PalletId};
use frame_system::EnsureRoot;
use pallet_collator_selection::{Config, IdentityCollator};
use pallet_xcm::{EnsureXcm, IsMajorityOfBody};
use xcm::latest::BodyId;
// --- darwinia-network ---
use crate::{weights::pallet_collator_selection::WeightInfo, *};

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
	type Currency = Ring;
	type Event = Event;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type MaxCandidates = MaxCandidates;
	type MaxInvulnerables = MaxInvulnerables;
	type MinCandidates = MinCandidates;
	type PotId = PotId;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = IdentityCollator;
	type ValidatorRegistration = Session;
	type WeightInfo = WeightInfo<Runtime>;
}
