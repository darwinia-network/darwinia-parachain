// --- parity ---
use frame_support::PalletId;
use frame_system::{EnsureOneOf, EnsureRoot};
use pallet_collator_selection::Config;
use pallet_xcm::{EnsureXcm, IsMajorityOfBody};
use xcm::v0::BodyId;
// --- darwinia ---
use crate::{weights::pallet_collator_selection::WeightInfo, *};

frame_support::parameter_types! {
	pub const ExecutiveBody: BodyId = BodyId::Executive;
	pub const PotId: PalletId = PalletId(*b"PotStake");
	pub const MaxCandidates: u32 = 1000;
	pub const MaxInvulnerables: u32 = 100;
}

/// We allow root and the Relay Chain council to execute privileged collator selection operations.
pub type CollatorSelectionUpdateOrigin = EnsureOneOf<
	AccountId,
	EnsureRoot<AccountId>,
	EnsureXcm<IsMajorityOfBody<KsmLocation, ExecutiveBody>>,
>;

impl Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type UpdateOrigin = CollatorSelectionUpdateOrigin;
	type PotId = PotId;
	type MaxCandidates = MaxCandidates;
	type MaxInvulnerables = MaxInvulnerables;
	// should be a multiple of session or things will get inconsistent
	type KickThreshold = Period;
	type WeightInfo = WeightInfo<Runtime>;
}
