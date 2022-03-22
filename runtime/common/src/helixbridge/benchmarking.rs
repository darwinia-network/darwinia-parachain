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

// --- crates.io ---
use array_bytes::hex2bytes_unchecked;
// --- paritytech ---
use frame_benchmarking::benchmarks;
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use frame_support::assert_ok;
use sp_runtime::traits::Zero;
// --- darwinia-network ---
use crate::*;

use crate::Pallet as ParaIssuing;

benchmarks! {
    issue_from_remote {
		let caller_bytes = hex2bytes_unchecked("0x28f900e9928c356287bb8806c9044168560dee80b91d2015653c9639a4998d8d");
		let caller: T::AccountId = T::AccountId::decode(&mut &caller_bytes[..]).unwrap_or_default();
        let addr_bytes = hex2bytes_unchecked("0x6d6f646c64612f73327362610000000000000000000000000000000000000000");
        let pallet_account_id: T::AccountId = T::AccountId::decode(&mut &addr_bytes[..]).unwrap_or_default();
        <T as Config>::RingCurrency::deposit_creating(&pallet_account_id, U256::from(5000).low_u128().saturated_into());
		let recipient_bytes = hex2bytes_unchecked("0x8e13b96a9c9e3b1832f07935be76c2b331251e26445f520ad1c56b24477ed8d6");
        let token_address = <T as Config>::RingAddress::get();
        assert_ok!(<ParaIssuing<T>>::set_remote_backing_account(
                RawOrigin::Root.into(),
                pallet_account_id
        ));
    }:_(RawOrigin::Signed(caller), token_address, recipient_bytes, 1000.into())

    burn_and_remote_unlock {
		let caller_bytes = hex2bytes_unchecked("0x28f900e9928c356287bb8806c9044168560dee80b91d2015653c9639a4998d8d");
		let caller: T::AccountId = T::AccountId::decode(&mut &caller_bytes[..]).unwrap_or_default();
        <T as Config>::RingCurrency::deposit_creating(&caller, U256::from(5000).low_u128().saturated_into());
        let recipient = caller.clone();
    }:_(RawOrigin::Signed(caller), 1, 1,
    100u128.saturated_into(),
    10u128.saturated_into(),
    recipient)

    set_remote_backing_account {
        let backing: T::AccountId = Default::default();
    }:_(RawOrigin::Root, backing)

	set_secure_limited_period {
		let period: BlockNumberFor<T> = Zero::zero();
	}:_(RawOrigin::Root, period)

	set_security_limitation_ring_amount {
		let limitation: RingBalance<T> = Zero::zero();
	}:_(RawOrigin::Root, limitation)
}
