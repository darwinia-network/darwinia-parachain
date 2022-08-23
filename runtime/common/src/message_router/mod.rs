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

//! Prototype module for message router.

// --- paritytech ---
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, Get, WithdrawReasons},
	weights::GetDispatchInfo,
};
pub use pallet::*;
use polkadot_parachain::primitives::Sibling;
use xcm::{
	prelude::*
};
use xcm_executor::traits::WeightBounds;
use xcm_builder::FixedWeightBounds;
use xcm_builder::SiblingParachainConvertsVia;
use xcm_executor::traits::Convert;

pub type AccountId<T> = <T as frame_system::Config>::AccountId;
pub type RingBalance<T> = <<T as Config>::RingCurrency as Currency<AccountId<T>>>::Balance;
pub type XcmUnitWeightCost = Weight;
pub type AssetUnitsPerSecond = u128;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::dispatch::PostDispatchInfo;
	use frame_support::weights::constants::WEIGHT_PER_SECOND;
	use super::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::Dispatchable;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type AssetModifierOrigin: EnsureOrigin<Self::Origin>;
		type Weigher: WeightBounds<Self::Call>;
		type RingCurrency: Currency<AccountId<Self>>;
		type MoonbeamLocation: Get<MultiLocation>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Changed the amount of units target chain charging per execution second for local asset
		TargetXcmExecConfigChanged {
			target_location: MultiLocation,
			local_asset_units_per_second: AssetUnitsPerSecond,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		TargetXcmExecNotConfig,
		WeightCalculationError,
		InsufficientBalance,
		AccountIdConversionFailed
	}

	/// Stores the units per second for target chain execution for local asset(e.g. CRAB).
	/// This is used to know how to charge for XCM execution in local asset.
	/// For example:
	/// key: 2023, val: 14719736222326895902025
	/// represents the units per second of CRAB token on moonriver
	#[pallet::storage]
	#[pallet::getter(fn target_xcm_exec_config)]
	pub type TargetXcmExecConfig<T: Config> =
	StorageMap<_, Blake2_128Concat, MultiLocation, AssetUnitsPerSecond>;

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(T::DbWeight::get().reads_writes(0, 1))]
		pub fn set_target_xcm_exec_config(
			origin: OriginFor<T>,
			target_location: MultiLocation,
			local_asset_units_per_second: AssetUnitsPerSecond,
		) -> DispatchResultWithPostInfo {
			T::AssetModifierOrigin::ensure_origin(origin)?;

			TargetXcmExecConfig::<T>::insert(&target_location,
				                            &local_asset_units_per_second
			                            );

			Self::deposit_event(Event::TargetXcmExecConfigChanged {
				target_location,
				local_asset_units_per_second,
			});

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn forward_to_moonbeam(origin: OriginFor<T>, mut message: Xcm<T::Call>)
		                           -> DispatchResultWithPostInfo {
			let user = ensure_signed(origin)?;

			let local_asset_units_per_second =
				TargetXcmExecConfig::<T>::get(T::MoonbeamLocation::get())
				.ok_or(Error::<T>::TargetXcmExecNotConfig)?;

			let mut amount:u128 = 0;
			let weight = T::Weigher::weight(&mut message)
				.map_err(|()| Error::<T>::WeightCalculationError)?;

			amount = local_asset_units_per_second.saturating_mul(weight as u128)
					/ (WEIGHT_PER_SECOND as u128);

			// Make sure the user's balance is enough to transfer
			ensure!(
				T::RingCurrency::free_balance(&user) > amount,
				Error::<T>::InsufficientBalance
			);

			let sovereign_account =
				SiblingParachainConvertsVia::<Sibling, dc_primitives::AccountId>::convert_ref(T::MoonbeamLocation::get())
				.map_err(|()| Error::AccountIdConversionFailed)?;

			// We need to transfer XCM execution fee from user to moonbeam sovereign account
			T::RingCurrency::transfer(
				&user,
				&sovereign_account,
				amount,
				ExistenceRequirement::KeepAlive,
			)?;

			Ok(().into())
		}
	}
}