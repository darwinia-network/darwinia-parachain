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
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

#![cfg(feature = "runtime-benchmarks")]

// --- paritytech ---
use frame_benchmarking::{account, benchmarks};
use frame_support::{assert_ok, traits::Get};
use frame_system::RawOrigin;
use sp_std::{boxed::Box, vec};
use xcm::{latest::prelude::*, VersionedXcm::V2};
// --- darwinia ---
use crate::message_router::{Call, Config, Pallet};
use dc_primitives::COIN;

benchmarks! {
	set_target_xcm_exec_config {
		let target_location = T::MoonbeamLocation::get();
		let local_asset_units_per_second: u128 = 14719736222326895902025_u128;
	}:_(RawOrigin::Root, target_location.clone(), local_asset_units_per_second)
	verify {
		assert_eq!(Pallet::<T>::target_xcm_exec_config(target_location), Some(local_asset_units_per_second));
	}

	forward_to_moonbeam {
		let target_location = T::MoonbeamLocation::get();
		let local_asset_units_per_second: u128 = 14719736222326895902025_u128;
		assert_ok!(Pallet::<T>::set_target_xcm_exec_config(RawOrigin::Root.into(), target_location.clone(), local_asset_units_per_second));
		let xcm = Xcm(vec![
			DescendOrigin(X1(AccountKey20 {
					network: Any,
					key: [0u8;20].into()
				})),
			]);
	}:_(RawOrigin::Root, Box::new(V2(xcm)))
}
