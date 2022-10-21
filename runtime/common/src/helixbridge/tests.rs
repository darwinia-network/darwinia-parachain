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

use crate::helixbridge::{mock::*, *};

use sp_std::str::FromStr;

// --- paritytech ---
use frame_support::{assert_err, assert_ok};
use frame_system::RawOrigin;

#[test]
fn issue_from_remote_backing_not_configured() {
	new_test_ext().execute_with(|| {
		let (recipient, _recipient_vec) = build_account(10);
		assert_err!(
			S2sIssuing::issue_from_remote(
				Origin::signed(build_account(1).0),
				1u64,
				recipient.clone(),
				vec![],
				0,
			),
			<Error<Test>>::BackingAccountNone
		);
		assert_eq!(Balances::free_balance(recipient), 0);
	});
}

#[test]
fn encode_evm_abi() {
	let unlock_bytes = evm::ToParachainBacking::encode_unlock_from_remote(
		H160::from_str("88a39B052d477CfdE47600a7C9950a441Ce61cb4").unwrap(),
		U256::from(10000000000000000000u128),
		vec![],
		1,
	)
	.unwrap();
	assert_eq!(
            hex::encode(unlock_bytes.clone()),
            "c1031ea300000000000000000000000088a39b052d477cfde47600a7c9950a441ce61cb40000000000000000000000000000000000000000000000008ac7230489e80000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000");
	let recv_bytes = evm::MessageEndpoint::encode_recv_message(unlock_bytes).unwrap();
	assert_eq!(
            hex::encode(recv_bytes.clone()),
            "b953c2e1000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000a4c1031ea300000000000000000000000088a39b052d477cfde47600a7c9950a441ce61cb40000000000000000000000000000000000000000000000008ac7230489e8000000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
}

#[test]
fn issue_from_remote_backing_remote_sender_invalid() {
	new_test_ext().execute_with(|| {
		let (recipient, _recipient_vec) = build_account(10);
		let (remote_backing_account, _) = build_account(3);
		assert_ok!(S2sIssuing::set_remote_backing_account(
			RawOrigin::Root.into(),
			remote_backing_account
		));
		assert_err!(
			S2sIssuing::issue_from_remote(
				Origin::signed(build_account(1).0),
				1u64,
				recipient.clone(),
				vec![],
				0,
			),
			BadOrigin
		);
		assert_eq!(Balances::free_balance(recipient), 0);
	});
}

#[test]
fn issue_from_remote_backing_success() {
	new_test_ext().execute_with(|| {
		let (recipient, _recipient_vec) = build_account(10);
		let (remote_backing_account, _) = build_account(3);
		let drived_remote_backing_account =
			S2sIssuing::derived_backing_id(remote_backing_account.clone());
		assert_ok!(S2sIssuing::set_remote_backing_account(
			RawOrigin::Root.into(),
			remote_backing_account
		));
		assert_ok!(S2sIssuing::issue_from_remote(
			Origin::signed(drived_remote_backing_account),
			1024u64,
			recipient.clone(),
			vec![],
			0,
		));
		assert_eq!(Balances::free_balance(recipient), 1_024);
	});
}

#[test]
fn burn_and_remote_unlock_insufficient_balance() {
	new_test_ext().execute_with(|| {
		assert_err!(
			S2sIssuing::burn_and_remote_unlock(
				Origin::signed(build_account(1).0),
				1,
				1,
				1000000,
				100,
				1,
				H160::from_str("1234500000000000000000000000000000000000").unwrap(),
			),
			<Error<Test>>::InsufficientBalance
		);
	})
}

#[test]
fn burn_and_remote_unlock_success() {
	new_test_ext().execute_with(|| {
		let (remote_backing_account, _) = build_account(3);
		assert_ok!(S2sIssuing::set_remote_backing_account(
			RawOrigin::Root.into(),
			remote_backing_account
		));
		assert_ok!(S2sIssuing::burn_and_remote_unlock(
			Origin::signed(build_account(1).0),
			1,
			1,
			1000000,
			10,
			1,
			H160::from_str("1234500000000000000000000000000000000000").unwrap(),
		));
		assert_eq!(Balances::free_balance(build_account(1).0), 89);
		assert_eq!(Balances::free_balance(S2sIssuing::pallet_account_id()), 0);
	})
}

#[test]
fn handle_issuing_failure_from_remote_success() {
	new_test_ext().execute_with(|| {
		let (remote_backing_account, _) = build_account(3);
		assert_ok!(S2sIssuing::set_remote_backing_account(
			RawOrigin::Root.into(),
			remote_backing_account.clone(),
		));
		// first lock and suppose target failed
		assert_ok!(S2sIssuing::burn_and_remote_unlock(
			Origin::signed(build_account(1).0),
			1,
			1,
			1000000,
			10,
			1,
			H160::from_str("1234500000000000000000000000000000000000").unwrap(),
		));
		MockS2sMessageSender::increase_outbound_nonce();
		assert_eq!(Balances::free_balance(build_account(1).0), 89);
		assert_eq!(Balances::free_balance(S2sIssuing::pallet_account_id()), 0);
		// unlock, the proof is verified on target
		let drived_remote_backing_account =
			S2sIssuing::derived_backing_id(remote_backing_account.clone());
		assert_ok!(S2sIssuing::handle_issuing_failure_from_remote(
			Origin::signed(drived_remote_backing_account),
			0,
			vec![],
			0,
		));
		assert_eq!(Balances::free_balance(build_account(1).0), 99);
		assert_eq!(Balances::free_balance(S2sIssuing::pallet_account_id()), 0);
	})
}

#[test]
fn handle_issuing_failure_from_remote_failed() {
	new_test_ext().execute_with(|| {
		let (remote_backing_account, _) = build_account(3);
		assert_ok!(S2sIssuing::set_remote_backing_account(
			RawOrigin::Root.into(),
			remote_backing_account.clone(),
		));
		// lock and suppose target failed
		assert_ok!(S2sIssuing::burn_and_remote_unlock(
			Origin::signed(build_account(1).0),
			1,
			1,
			1000000,
			10,
			1,
			H160::from_str("1234500000000000000000000000000000000000").unwrap(),
		));
		MockS2sMessageSender::increase_outbound_nonce();
		// lock and suppose target failed
		assert_ok!(S2sIssuing::burn_and_remote_unlock(
			Origin::signed(build_account(1).0),
			1,
			1,
			1000000,
			10,
			1,
			H160::from_str("1234500000000000000000000000000000000000").unwrap(),
		));
		MockS2sMessageSender::increase_outbound_nonce();
		assert_eq!(Balances::free_balance(build_account(1).0), 78);
		assert_eq!(Balances::free_balance(S2sIssuing::pallet_account_id()), 0);
		// unlock, the proof is verified on target
		let drived_remote_backing_account =
			S2sIssuing::derived_backing_id(remote_backing_account.clone());
		assert_ok!(S2sIssuing::handle_issuing_failure_from_remote(
			Origin::signed(drived_remote_backing_account.clone()),
			0,
			vec![],
			0,
		));
		MockS2sMessageSender::increase_inbound_nonce();
		assert_ok!(S2sIssuing::handle_issuing_failure_from_remote(
			Origin::signed(drived_remote_backing_account.clone()),
			1,
			vec![],
			0,
		));
		MockS2sMessageSender::increase_inbound_nonce();
		assert_eq!(Balances::free_balance(build_account(1).0), 98);
		assert_eq!(Balances::free_balance(S2sIssuing::pallet_account_id()), 0);
		// test failed
		// duplicate redeem
		assert_err!(
			S2sIssuing::handle_issuing_failure_from_remote(
				Origin::signed(drived_remote_backing_account.clone()),
				1,
				vec![],
				0,
			),
			<Error<Test>>::FailureInfoNE,
		);
		MockS2sMessageSender::increase_inbound_nonce();
		// not exist
		assert_err!(
			S2sIssuing::handle_issuing_failure_from_remote(
				Origin::signed(drived_remote_backing_account.clone()),
				2,
				vec![],
				0,
			),
			<Error<Test>>::FailureInfoNE,
		);
		MockS2sMessageSender::increase_inbound_nonce();
		// test local refund
		assert_ok!(S2sIssuing::burn_and_remote_unlock(
			Origin::signed(build_account(1).0),
			1,
			1,
			1000000,
			10,
			1,
			H160::from_str("1234500000000000000000000000000000000000").unwrap(),
		));
		MockS2sMessageSender::increase_outbound_nonce();
		assert_eq!(Balances::free_balance(build_account(1).0), 87);
		assert_eq!(Balances::free_balance(S2sIssuing::pallet_account_id()), 0);
		assert_ok!(S2sIssuing::burn_and_remote_unlock(
			Origin::signed(build_account(1).0),
			1,
			1,
			1000000,
			10,
			1,
			H160::from_str("1234500000000000000000000000000000000000").unwrap(),
		));
		MockS2sMessageSender::increase_outbound_nonce();
		assert_eq!(Balances::free_balance(build_account(1).0), 76);
		assert_eq!(Balances::free_balance(S2sIssuing::pallet_account_id()), 0);
		assert_ok!(S2sIssuing::burn_and_remote_unlock(
			Origin::signed(build_account(1).0),
			1,
			1,
			1000000,
			10,
			1,
			H160::from_str("1234500000000000000000000000000000000000").unwrap(),
		));
		MockS2sMessageSender::increase_outbound_nonce();
		assert_eq!(Balances::free_balance(build_account(1).0), 65);
		assert_eq!(Balances::free_balance(S2sIssuing::pallet_account_id()), 0);
		assert_ok!(S2sIssuing::issue_from_remote(
			Origin::signed(drived_remote_backing_account),
			100u64,
			build_account(1).0,
			vec![4],
			0,
		));
		MockS2sMessageSender::increase_inbound_nonce();
		assert_eq!(Balances::free_balance(build_account(1).0), 165);
		assert_eq!(Balances::free_balance(S2sIssuing::pallet_account_id()), 0);
		// failed
		assert_ok!(S2sIssuing::handle_issuing_failure_local(Origin::signed(build_account(1).0), 3));
		assert_err!(
			S2sIssuing::handle_issuing_failure_local(Origin::signed(build_account(1).0), 4),
			<Error<Test>>::FailureInfoNE
		);
		assert_err!(
			S2sIssuing::handle_issuing_failure_local(Origin::signed(build_account(1).0), 5),
			<Error<Test>>::FailureNonceInvalid
		);
		assert_eq!(Balances::free_balance(build_account(1).0), 175);
		assert_eq!(Balances::free_balance(S2sIssuing::pallet_account_id()), 0);
	})
}

#[test]
fn prun_message() {
	new_test_ext().execute_with(|| {
		let (remote_backing_account, _) = build_account(3);
		let (recipient, _recipient_vec) = build_account(10);
		assert_ok!(S2sIssuing::set_remote_backing_account(
			RawOrigin::Root.into(),
			remote_backing_account.clone(),
		));
		assert_ok!(S2sIssuing::burn_and_remote_unlock(
			Origin::signed(build_account(1).0),
			1,
			1,
			1000000,
			10,
			1,
			H160::from_str("1234500000000000000000000000000000000000").unwrap(),
		));
		MockS2sMessageSender::increase_outbound_nonce();
		assert_ok!(S2sIssuing::burn_and_remote_unlock(
			Origin::signed(build_account(1).0),
			1,
			1,
			1000000,
			10,
			1,
			H160::from_str("1234500000000000000000000000000000000000").unwrap(),
		));
		MockS2sMessageSender::increase_outbound_nonce();
		assert_ok!(S2sIssuing::burn_and_remote_unlock(
			Origin::signed(build_account(1).0),
			1,
			1,
			1000000,
			10,
			1,
			H160::from_str("1234500000000000000000000000000000000000").unwrap(),
		));
		MockS2sMessageSender::increase_outbound_nonce();
		assert_eq!(Balances::free_balance(build_account(1).0), 67);
		assert_eq!(Balances::free_balance(S2sIssuing::pallet_account_id()), 0);
		// unlock, the proof is verified on target
		let drived_remote_backing_account =
			S2sIssuing::derived_backing_id(remote_backing_account.clone());
		assert_ok!(S2sIssuing::issue_from_remote(
			Origin::signed(drived_remote_backing_account.clone()),
			1u64,
			recipient.clone(),
			vec![],
			0,
		),);
		MockS2sMessageSender::increase_inbound_nonce();
		assert_ok!(S2sIssuing::issue_from_remote(
			Origin::signed(drived_remote_backing_account.clone()),
			1u64,
			recipient.clone(),
			vec![],
			0,
		),);
		MockS2sMessageSender::increase_inbound_nonce();

		// out: 0, 1, 2
		// in: 0, 1
		// suppose out message 1 failed
		assert_ok!(S2sIssuing::issue_from_remote(
			Origin::signed(drived_remote_backing_account.clone()),
			1u64,
			recipient.clone(),
			vec![0, 2],
			1,
		),);
		// out: 1
		// in: 1, 2
		MockS2sMessageSender::increase_inbound_nonce();
		assert_err!(
			S2sIssuing::handle_issuing_failure_from_remote(
				Origin::signed(drived_remote_backing_account.clone()),
				0,
				vec![],
				0,
			),
			<Error<Test>>::FailureInfoNE,
		);
		MockS2sMessageSender::increase_inbound_nonce();
		assert_err!(
			S2sIssuing::handle_issuing_failure_from_remote(
				Origin::signed(drived_remote_backing_account.clone()),
				2,
				vec![],
				0,
			),
			<Error<Test>>::FailureInfoNE,
		);
		// in: 1, 2 (3, 4 failed)
		MockS2sMessageSender::increase_inbound_nonce();
		assert_ok!(S2sIssuing::handle_issuing_failure_from_remote(
			Origin::signed(drived_remote_backing_account.clone()),
			1,
			vec![],
			3,
		),);
		MockS2sMessageSender::increase_inbound_nonce();
		assert_ok!(S2sIssuing::issue_from_remote(
			Origin::signed(drived_remote_backing_account.clone()),
			10,
			recipient.clone(),
			vec![],
			0,
		));
		// in: 6, 7
		assert_ok!(S2sIssuing::remote_unlock_failure(
			Origin::signed(build_account(1).0),
			1,
			1,
			1000000,
			1,
			1,
		),);
		assert_ok!(S2sIssuing::remote_unlock_failure(
			Origin::signed(build_account(1).0),
			1,
			1,
			1000000,
			2,
			1,
		),);
		assert_err!(
			S2sIssuing::remote_unlock_failure(
				Origin::signed(build_account(1).0),
				1,
				1,
				1000000,
				6,
				1,
			),
			<Error<Test>>::MessageAlreadyIssued,
		);
		assert_err!(
			S2sIssuing::remote_unlock_failure(
				Origin::signed(build_account(1).0),
				1,
				1,
				1000000,
				7,
				1,
			),
			<Error<Test>>::MessageAlreadyIssued,
		);
		assert_err!(
			S2sIssuing::remote_unlock_failure(
				Origin::signed(build_account(1).0),
				1,
				1,
				1000000,
				8,
				1,
			),
			<Error<Test>>::MessageNotDelived,
		);
	})
}
