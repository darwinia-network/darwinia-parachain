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

//! Unit tests for the remote governance module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn test_accept_remote_call() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		let force_balance_transfer =
			Box::new(mock::Call::Balances(pallet_balances::Call::<Test>::force_transfer {
				source: ALICE,
				dest: BOB,
				value: 11,
			}));
		assert_noop!(
			RemoteGovernance::accept_remote_call(
				Origin::signed(ALICE_SLASH),
				force_balance_transfer.clone()
			),
			Error::<Test>::RequireSourceRoot
		);

		let source_root = RemoteGovernance::derived_source_root();
		assert_ok!(RemoteGovernance::accept_remote_call(
			Origin::signed(source_root),
			force_balance_transfer
		));
	});
}

#[test]
fn test_rescue_call() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);

		let force_balance_transfer =
			Box::new(mock::Call::Balances(pallet_balances::Call::<Test>::force_transfer {
				source: ALICE,
				dest: BOB,
				value: 11,
			}));
		assert_noop!(
			RemoteGovernance::rescue_call(
				Origin::signed(ALICE_SLASH),
				force_balance_transfer.clone()
			),
			Error::<Test>::RequireRescuer
		);

		assert_ok!(RemoteGovernance::rescue_call(
			Origin::signed(BOB_SLASH),
			force_balance_transfer
		));
		assert_eq!(Balances::free_balance(BOB), 11);
	});
}
