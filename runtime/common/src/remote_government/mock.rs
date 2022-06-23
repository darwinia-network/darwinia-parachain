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

mod remote_government {
	pub use super::super::*;
}

// --- paritytech ---
use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU32, Everything, OnInitialize},
};
use frame_system::mocking::*;
use pallet_balances::AccountData;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};
// --- darwinia-network ---
use super::*;

pub(crate) type AccountId = AccountId32;
pub(crate) type BlockNumber = u64;
pub(crate) type Balance = u128;

pub(crate) const ALICE: AccountId = AccountId32::new([0; 32]);
pub(crate) const BOB: AccountId = AccountId32::new([1; 32]);

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}
impl frame_system::Config for Test {
	type AccountData = AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = Everything;
	type BlockHashCount = BlockHashCount;
	type BlockLength = ();
	type BlockNumber = BlockNumber;
	type BlockWeights = ();
	type Call = Call;
	type DbWeight = ();
	type Event = Event;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<AccountId>;
	type MaxConsumers = ConstU32<16>;
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

pub struct EnsureAlice;
impl<O: Into<Result<RawOrigin<AccountId>, O>> + From<RawOrigin<AccountId>>> EnsureOrigin<O>
	for EnsureAlice
{
	type Success = AccountId;

	fn try_origin(o: O) -> Result<Self::Success, O> {
		o.into().and_then(|o| match o {
			RawOrigin::Signed(who) if who == ALICE => Ok(who),
			r => Err(O::from(r)),
		})
	}
}

frame_support::parameter_types! {
	pub const PangolinChainId: bp_runtime::ChainId = *b"pagl";
	pub const CheckInterval: BlockNumber = 3;
}
impl Config for Test {
	type BridgeAccountIdConverter = AccountIdConverter;
	// type BridgeFinalized = ();
	type BridgedChainId = PangolinChainId;
	type Call = Call;
	// type CheckInterval = CheckInterval;
	type EmergencySafeguardOrigin = EnsureAlice;
	type Event = Event;
}

construct_runtime!(
	pub enum Test where
		Block = MockBlock<Test>,
		NodeBlock = MockBlock<Test>,
		UncheckedExtrinsic = MockUncheckedExtrinsic<Test>
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Storage, Call, Event<T>},
		RemoteGovernment: remote_government::{Pallet, Storage, Call, Event<T>},
	}
);

pub(crate) struct ExtBuilder;
impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		let balances = vec![(ALICE, 1000)];
		pallet_balances::GenesisConfig::<Test> { balances }.assimilate_storage(&mut t).unwrap();

		t.into()
	}
}
impl Default for ExtBuilder {
	fn default() -> Self {
		ExtBuilder
	}
}

pub fn run_to_block(n: BlockNumber) {
	for b in System::block_number() + 1..=n {
		System::set_block_number(b);
		<remote_government::Pallet<Test> as OnInitialize<BlockNumber>>::on_initialize(b);
	}
}
