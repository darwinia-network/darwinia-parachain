// --- parity ---
use frame_support::traits::Currency;
use frame_system::Config as SystemConfig;
use pallet_balances::{Config, Pallet};
// --- darwinia ---
use crate::{weights::pallet_balances::WeightInfo, *};

pub type NegativeImbalance<R> =
	<Pallet<R> as Currency<<R as SystemConfig>::AccountId>>::NegativeImbalance;

frame_support::parameter_types! {
	pub const ExistentialDeposit: Balance = 0;
	pub const MaxLocks: u32 = 50;
}

impl Config for Runtime {
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = MaxLocks;
	type WeightInfo = WeightInfo<Runtime>;
}
