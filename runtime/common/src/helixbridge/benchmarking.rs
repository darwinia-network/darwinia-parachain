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
use frame_benchmarking::benchmarks;
use frame_support::assert_ok;
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_runtime::traits::Zero;
// --- darwinia-network ---
use codec::Decode;
use ethereum_types::H160;
use sp_std::str::FromStr;

use crate::helixbridge::{Pallet as ParaIssuing, *};

pub fn build_account<AccountId: Decode>(x: u8) -> AccountId {
	let origin = [x; 32];
	AccountId::decode(&mut &origin[..]).unwrap()
}

benchmarks! {
	issue_from_remote {
		let remote_backing = build_account::<T::AccountId>(1);
		let recipient = build_account::<T::AccountId>(2);
		assert_ok!(<ParaIssuing<T>>::set_remote_backing_account(
				RawOrigin::Root.into(),
				remote_backing.clone()
		));
		let caller = <ParaIssuing<T>>::derived_backing_id(remote_backing.clone());
	}:_(RawOrigin::Signed(caller), 1000u128.saturated_into(), recipient, vec![], 0)

	handle_issuing_failure_from_remote {
		let remote_backing = build_account::<T::AccountId>(1);
		let recipient = build_account::<T::AccountId>(2);
		assert_ok!(<ParaIssuing<T>>::set_remote_backing_account(
				RawOrigin::Root.into(),
				remote_backing.clone()
		));
		let caller = <ParaIssuing<T>>::derived_backing_id(remote_backing.clone());
		let message_id = (T::MessageLaneId::get(), 1);
		let value: RingBalance<T> = Zero::zero();
		TransactionInfos::<T>::insert(message_id, (caller.clone(), value));
	}:_(RawOrigin::Signed(caller), 1, vec![], 0)

	burn_and_remote_unlock {
		let remote_backing = build_account::<T::AccountId>(1);
		assert_ok!(<ParaIssuing<T>>::set_remote_backing_account(
				RawOrigin::Root.into(),
				remote_backing.clone()
		));
		let recipient = H160::from_str("1234500000000000000000000000000000000000").unwrap();
		let caller = build_account::<T::AccountId>(2);
		<T as Config>::RingCurrency::deposit_creating(&caller, U256::from(5000).low_u128().saturated_into());
	}:_(RawOrigin::Signed(caller), 1, 1,
	1000000,
	100u128.saturated_into(),
	10u128.saturated_into(),
	recipient)

	remote_unlock_failure {
		let remote_backing = build_account::<T::AccountId>(1);
		assert_ok!(<ParaIssuing<T>>::set_remote_backing_account(
				RawOrigin::Root.into(),
				remote_backing.clone()
		));
		let caller = build_account::<T::AccountId>(2);
		<T as Config>::RingCurrency::deposit_creating(&caller, U256::from(5000).low_u128().saturated_into());
		//ReceivedNonces::<T>::try_mutate(|nonces| -> DispatchResult {
			//nonces.try_push(1).map_err(|_| <Error<T>>::TooManyNonces)?;
			//Ok(())
		//})?;
	}:_(RawOrigin::Signed(caller), 1, 1, 1000000, 0, 10u128.saturated_into())

	set_remote_backing_account {
		let remote_backing = build_account::<T::AccountId>(1);
	}:_(RawOrigin::Root, remote_backing)

	set_secure_limited_period {
		let period: BlockNumberFor<T> = Zero::zero();
	}:_(RawOrigin::Root, period)

	set_security_limitation_ring_amount {
		let limitation: RingBalance<T> = Zero::zero();
	}:_(RawOrigin::Root, limitation)
}
