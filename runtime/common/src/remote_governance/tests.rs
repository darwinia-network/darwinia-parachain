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

// --- paritytech ---
use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError::BadOrigin;
// --- darwinia-network ---
use super::{
	mock::{Call, *},
	*,
};

#[test]
fn emergency_safeguard_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		run_to_block(1);

		let force_balance_transfer =
			Box::new(Call::Balances(pallet_balances::Call::<Test>::force_transfer {
				source: ALICE,
				dest: BOB,
				value: 11,
			}));

		assert_noop!(
			RemoteGovernance::emergency_safeguard(
				Origin::signed(ALICE),
				force_balance_transfer.clone()
			),
			<Error<Test>>::EmergencyOnly
		);
		assert_eq!(Balances::free_balance(BOB), 0);

		run_to_block(4);

		assert_noop!(
			RemoteGovernance::emergency_safeguard(
				Origin::signed(BOB),
				force_balance_transfer.clone()
			),
			BadOrigin
		);

		assert_ok!(RemoteGovernance::emergency_safeguard(
			Origin::signed(ALICE),
			force_balance_transfer
		));
		assert_eq!(Balances::free_balance(BOB), 11);
	});
}

#[test]
fn enact_remote_call_should_work() {
	ExtBuilder::default().build().execute_with(|| {
		run_to_block(1);

		let force_balance_transfer =
			Box::new(Call::Balances(pallet_balances::Call::<Test>::force_transfer {
				source: ALICE,
				dest: BOB,
				value: 11,
			}));

		assert_noop!(
			RemoteGovernance::enact_remote_call(
				Origin::signed(ALICE),
				force_balance_transfer.clone()
			),
			<Error<Test>>::RequireSourceRoot
		);
		assert_eq!(Balances::free_balance(BOB), 0);

		let source_root = RemoteGovernance::derived_source_root();

		assert_ok!(RemoteGovernance::enact_remote_call(
			Origin::signed(source_root),
			force_balance_transfer
		));
		assert_eq!(Balances::free_balance(BOB), 11);
	});
}
