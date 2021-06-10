// --- parity ---
use frame_support::traits::Currency;
use frame_system::Config as SystemConfig;
use pallet_balances::{Config, Pallet};
// --- darwinia ---
use crate::{weights::pallet_balances::WeightInfo, *};

pub type NegativeImbalance =
	<Pallet<Runtime> as Currency<<Runtime as SystemConfig>::AccountId>>::NegativeImbalance;

frame_support::parameter_types! {
	pub const ExistentialDeposit: Balance = 0;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl Config for Runtime {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = WeightInfo<Runtime>;
}
