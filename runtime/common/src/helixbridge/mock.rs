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
//

// --- paritytech ---
use bp_messages::source_chain::SendMessageArtifacts;
use frame_support::{
	traits::{Everything, GenesisBuild},
	PalletId,
};
use frame_system::mocking::*;
use pallet_balances::AccountData;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};
// --- darwinia-network ---
use crate::helixbridge::{
	*, {self as s2s_issuing},
};

type Block = MockBlock<Test>;
//type SignedExtra = (frame_system::CheckSpecVersion<Test>,);
type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test, (), ()>;
type Balance = u64;

pub fn build_account(x: u8) -> (AccountId32, Vec<u8>) {
	let origin = [x; 32];
	(AccountId32::decode(&mut &origin.clone()[..]).unwrap(), origin.to_vec())
}

frame_support::parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type Event = ();
	type ExistentialDeposit = ExistentialDeposit;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

frame_support::parameter_types! {
	pub const MinimumPeriod: u64 = 6000 / 2;
}
impl pallet_timestamp::Config for Test {
	type MinimumPeriod = MinimumPeriod;
	type Moment = u64;
	type OnTimestampSet = ();
	type WeightInfo = ();
}

impl frame_system::Config for Test {
	type AccountData = AccountData<Balance>;
	type AccountId = AccountId32;
	type BaseCallFilter = Everything;
	type BlockHashCount = ();
	type BlockLength = ();
	type BlockNumber = u64;
	type BlockWeights = ();
	type Call = Call;
	type DbWeight = ();
	type Event = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
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

frame_support::parameter_types! {
	pub const S2sRelayPalletId: PalletId = PalletId(*b"da/s2sre");
	pub const PangolinChainId: bp_runtime::ChainId = *b"pagl";
	pub PangolinName: Vec<u8> = (b"Pangolin").to_vec();
	pub MessageLaneId: [u8; 4] = *b"ptol";
	pub DecimalsDifference: Balance = 1_000_000_000;
}

pub struct AccountIdConverter;
impl Convert<H256, AccountId32> for AccountIdConverter {
	fn convert(hash: H256) -> AccountId32 {
		hash.to_fixed_bytes().into()
	}
}
pub struct MockS2sMessageSender;
impl LatestMessageNoncer for MockS2sMessageSender {
	fn outbound_latest_generated_nonce(_lane_id: [u8; 4]) -> u64 {
		0
	}
}

pub struct MockMessagesBridge;
impl MessagesBridge<Origin, AccountId<Test>, Balance, ()> for MockMessagesBridge {
	type Error = DispatchErrorWithPostInfo<PostDispatchInfo>;

	fn send_message(
		submitter: Origin,
		_laneid: [u8; 4],
		_payload: (),
		fee: Balance,
	) -> Result<SendMessageArtifacts, Self::Error> {
		// send fee to fund account [2;32]
		Balances::transfer(submitter.into(), build_account(2).0, fee)?;
		Ok(SendMessageArtifacts { nonce: 0, weight: 0 })
	}
}

impl<AccountId, Signer, Signature> CreatePayload<AccountId, Signer, Signature, Test> for () {
	type Payload = ();

	fn create(
		_: CallOrigin<AccountId, Signer, Signature>,
		_: u32,
		_: u64,
		_: CallParams<Test>,
		_: DispatchFeePayment,
	) -> Result<Self::Payload, &'static str> {
		Ok(())
	}
}

impl Config for Test {
	type BridgedAccountIdConverter = AccountIdConverter;
	type BridgedChainId = PangolinChainId;
	type DecimalMultiplier = DecimalsDifference;
	type Event = ();
	type MessageLaneId = MessageLaneId;
	type MessageNoncer = MockS2sMessageSender;
	type MessagesBridge = MockMessagesBridge;
	type OutboundPayloadCreator = ();
	type PalletId = S2sRelayPalletId;
	type RingCurrency = Balances;
	type WeightInfo = ();
}

frame_support::construct_runtime! {
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>} = 0,
		Timestamp: pallet_timestamp::{Pallet, Call, Inherent, Storage} = 1,
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>} = 2,
		S2sIssuing: s2s_issuing::{Pallet, Call, Storage, Config<T>, Event<T>} = 3,
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();

	s2s_issuing::GenesisConfig::<Test> {
		secure_limited_period: 10,
		secure_limited_ring_amount: 1_000_000_000_000_000,
	}
	.assimilate_storage(&mut storage)
	.unwrap();

	// add some balance to backing account 10 ring
	let balances = vec![(build_account(1).0, 100)];
	pallet_balances::GenesisConfig::<Test> { balances }.assimilate_storage(&mut storage).unwrap();

	storage.into()
}
