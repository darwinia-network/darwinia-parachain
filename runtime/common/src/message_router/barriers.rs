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

//! Various implementations for `ShouldExecute`.

use frame_support::{ensure, log, traits::Contains, weights::Weight};
use sp_std::{marker::PhantomData, result::Result};
use xcm::latest::{
	Instruction::*, MultiLocation, WeightLimit::*, Xcm,
};
use xcm_executor::traits::ShouldExecute;

/// Allows execution from `origin` if it is contained in `T` (i.e. `T::Contains(origin)`) taking
/// payments into account.
///
/// Only allows for `TeleportAsset`, `WithdrawAsset`, `ClaimAsset` and `ReserveAssetDeposit` XCMs
/// because they are the only ones that place assets in the Holding Register to pay for execution.
pub struct AllowDescendOriginPaidExecutionFrom<T>(PhantomData<T>);
impl<T: Contains<MultiLocation>> ShouldExecute for AllowDescendOriginPaidExecutionFrom<T> {
	fn should_execute<Call>(
		origin: &MultiLocation,
		message: &mut Xcm<Call>,
		max_weight: Weight,
		_weight_credit: &mut Weight,
	) -> Result<(), ()> {
		log::trace!(
			target: "xcm::barriers",
			"AllowDescendOriginPaidExecutionFrom origin: {:?}, message: {:?}, max_weight: {:?}, weight_credit: {:?}",
			origin, message, max_weight, _weight_credit,
		);
		ensure!(T::contains(origin), ());
		let mut iter = message.0.iter_mut();
		let i = iter.next().ok_or(())?;
		match i {
			DescendOrigin(_) => (),
			_ => return Err(()),
		}
		let i = iter.next().ok_or(())?;
		match i {
			ReceiveTeleportedAsset(..)
			| WithdrawAsset(..)
			| ReserveAssetDeposited(..)
			| ClaimAsset { .. } => (),
			_ => return Err(()),
		}
		let mut i = iter.next().ok_or(())?;
		while let ClearOrigin = i {
			i = iter.next().ok_or(())?;
		}
		match i {
			BuyExecution { weight_limit: Limited(ref mut weight), .. } if *weight >= max_weight => {
				*weight = max_weight;
				Ok(())
			},
			BuyExecution { ref mut weight_limit, .. } if weight_limit == &Unlimited => {
				*weight_limit = Limited(max_weight);
				Ok(())
			},
			_ => Err(()),
		}
	}
}
