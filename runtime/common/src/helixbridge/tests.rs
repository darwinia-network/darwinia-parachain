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
			),
			<Error<Test>>::BackingAccountNone
		);
		assert_eq!(Balances::free_balance(recipient), 0);
	});
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
		));
		assert_eq!(Balances::free_balance(recipient), 1_024_000_000_000);
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
				100,
				1,
				build_account(1).0,
			),
			<Error<Test>>::InsufficientBalance
		);
	})
}

#[test]
fn burn_and_remote_unlock_success() {
	new_test_ext().execute_with(|| {
		assert_ok!(S2sIssuing::burn_and_remote_unlock(
			Origin::signed(build_account(1).0),
			1,
			1,
			10,
			1,
			build_account(1).0,
		));
		assert_eq!(Balances::free_balance(build_account(1).0), 89);
		assert_eq!(Balances::free_balance(S2sIssuing::pallet_account_id()), 10);
	})
}
