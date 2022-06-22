// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

//! Mocks for the governance module.

#![cfg(test)]

use super::*;
use frame_support::{construct_runtime, parameter_types, traits::Everything};
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup, AccountId32};

pub type AccountId = AccountId32;
pub type Balance = u128;
pub const ALICE: AccountId = AccountId32::new([0; 32]);
pub const BOB: AccountId = AccountId32::new([1; 32]);
pub const ALICE_SLASH: AccountId = AccountId32::new([2; 32]);
pub const BOB_SLASH: AccountId = AccountId32::new([3; 32]);

mod remote_governance {
	pub use super::super::*;
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = Everything;
	type BlockHashCount = BlockHashCount;
	type BlockLength = ();
	type BlockNumber = u64;
	type BlockWeights = ();
	type Call = Call;
	type DbWeight = ();
	type Event = Event;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<AccountId>;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type Origin = Origin;
	type PalletInfo = PalletInfo;
	type SS58Prefix = ();
	type SystemWeightInfo = ();
	type Version = ();
}

parameter_types! {
	pub const NativeTokenExistentialDeposit: Balance = 10;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = NativeTokenExistentialDeposit;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

pub struct AccountIdConverter;
impl Convert<H256, AccountId32> for AccountIdConverter {
	fn convert(hash: H256) -> AccountId32 {
		hash.to_fixed_bytes().into()
	}
}

pub struct RescuerBobSlash;
impl Rescuer<Origin> for RescuerBobSlash {
	fn allow(origin: Origin) -> bool {
		match ensure_signed::<Origin, AccountId>(origin.into()) {
			Ok(user) => user == BOB_SLASH,
			_ => false,
		}
	}
}

frame_support::parameter_types! {
	pub const PangolinChainId: bp_runtime::ChainId = *b"pagl";
}

impl Config for Test {
	type BridgedAccountIdConverter = AccountIdConverter;
	type BridgedChainId = PangolinChainId;
	type Call = Call;
	type Event = Event;
	type Rescuer = RescuerBobSlash;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		RemoteGovernance: remote_governance::{Pallet, Storage, Call, Event<T>},
		Balances: pallet_balances::{Pallet, Storage, Call, Event<T>},
	}
);

pub struct ExtBuilder;

impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		let balances = vec![(ALICE, 1000)];
		pallet_balances::GenesisConfig::<Test> { balances }.assimilate_storage(&mut t).unwrap();

		t.into()
	}
}
