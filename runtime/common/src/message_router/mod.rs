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
use frame_support::{pallet_prelude::*, traits::Get};
pub use pallet::*;
use xcm::prelude::*;
use xcm_executor::traits::WeightBounds;

pub type AssetUnitsPerSecond = u128;

pub trait ConvertXcm<Call> {
	fn convert(xcm: Xcm<()>) -> Option<Xcm<Call>>;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{log, weights::constants::WEIGHT_PER_SECOND};
	use frame_system::pallet_prelude::*;
	use sp_std::{boxed::Box, vec};
	use xcm_executor::traits::{InvertLocation, TransactAsset};

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type AssetModifierOrigin: EnsureOrigin<Self::Origin>;
		type MoonbeamWeigher: WeightBounds<Self::Call>;
		type LocalWeigher: WeightBounds<Self::Call>;
		type LocalAssetId: Get<MultiLocation>;
		type LocationInverter: InvertLocation;
		type SelfLocationInSibl: Get<MultiLocation>;
		type AssetTransactor: TransactAsset;
		type MoonbeamLocation: Get<MultiLocation>;
		type ExecuteXcmOrigin: EnsureOrigin<
			<Self as frame_system::Config>::Origin,
			Success = MultiLocation,
		>;
		type XcmExecutor: ExecuteXcm<Self::Call>;
		type XcmSender: SendXcm;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Changed the amount of units target chain charging per execution second for local asset
		TargetXcmExecConfigChanged {
			target_location: MultiLocation,
			local_asset_units_per_second: AssetUnitsPerSecond,
		},
		Route(MultiLocation, Xcm<()>, Weight, u128),
	}

	#[pallet::error]
	pub enum Error<T> {
		TargetXcmExecNotConfig,
		/// The message's weight could not be determined.
		UnweighableMessage,
		/// XCM execution failed. https://github.com/paritytech/substrate/pull/10242
		XcmExecutionFailed,
		BadVersion,
		/// MultiLocation value too large to descend further.
		MultiLocationFull,
		/// Failed to send xcm.
		XcmSendFailed,
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

			TargetXcmExecConfig::<T>::insert(&target_location, &local_asset_units_per_second);

			Self::deposit_event(Event::TargetXcmExecConfigChanged {
				target_location,
				local_asset_units_per_second,
			});

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn forward_to_moonbeam(
			origin: OriginFor<T>,
			message: Box<VersionedXcm<<T as frame_system::Config>::Call>>,
		) -> DispatchResultWithPostInfo {
			let origin_location = T::ExecuteXcmOrigin::ensure_origin(origin)?;
			let remote_xcm: Xcm<<T as frame_system::Config>::Call> =
				(*message).try_into().map_err(|()| Error::<T>::BadVersion)?;

			// Calculate the execution fee required to execute remote xcm
			let local_asset_units_per_second =
				TargetXcmExecConfig::<T>::get(T::MoonbeamLocation::get())
					.ok_or(Error::<T>::TargetXcmExecNotConfig)?;
			let remote_weight = T::MoonbeamWeigher::weight(
				&mut Self::extend_remote_xcm_for_weight(remote_xcm.clone()),
			)
			.map_err(|()| Error::<T>::UnweighableMessage)?;
			let amount = local_asset_units_per_second.saturating_mul(remote_weight as u128)
				/ (WEIGHT_PER_SECOND as u128);
			let remote_xcm_fee =
				MultiAsset { id: AssetId::from(T::LocalAssetId::get()), fun: Fungible(amount) };

			// Transfer xcm execution fee to moonbeam sovereign account
			let mut local_xcm = Xcm(vec![TransferAsset {
				assets: remote_xcm_fee.clone().into(),
				beneficiary: T::MoonbeamLocation::get(),
			}]);
			let local_weight = T::LocalWeigher::weight(&mut local_xcm)
				.map_err(|()| Error::<T>::UnweighableMessage)?;
			T::XcmExecutor::execute_xcm_in_credit(
				origin_location.clone(),
				local_xcm,
				local_weight,
				local_weight,
			)
			.ensure_complete()
			.map_err(|error| {
				log::error!("Failed execute route message with {:?}", error);
				Error::<T>::XcmExecutionFailed
			})?;

			// Extend remote xcm to buy execution and handle error remotely
			let ancestry = T::LocationInverter::ancestry();
			let mut remote_xcm_fee_anchor_dest = remote_xcm_fee.clone();
			remote_xcm_fee_anchor_dest
				.reanchor(&T::MoonbeamLocation::get(), &ancestry)
				.map_err(|()| Error::<T>::MultiLocationFull)?;
			let mut extend_remote_xcm = Xcm(vec![
				ReserveAssetDeposited(remote_xcm_fee_anchor_dest.clone().into()),
				BuyExecution {
					fees: remote_xcm_fee_anchor_dest,
					weight_limit: WeightLimit::Unlimited,
				},
				SetAppendix(Xcm(vec![
					RefundSurplus,
					DepositAsset {
						assets: Wild(All),
						max_assets: 1,
						beneficiary: T::SelfLocationInSibl::get(),
					},
				])),
			]);
			extend_remote_xcm.0.extend(remote_xcm.0.into_iter());

			// Send remote xcm to moonbeam
			T::XcmSender::send_xcm(T::MoonbeamLocation::get(), extend_remote_xcm.clone().into())
				.map_err(|_| Error::<T>::XcmSendFailed)?;

			Self::deposit_event(Event::Route(
				origin_location,
				extend_remote_xcm.into(),
				remote_weight,
				amount,
			));
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Extend xcm to make weight calculation more accurate.
		/// These extended instructions are the instructions that will be executed remotely
		fn extend_remote_xcm_for_weight(
			xcm: Xcm<<T as frame_system::Config>::Call>,
		) -> Xcm<<T as frame_system::Config>::Call> {
			let mut extend_xcm_for_weight = Xcm(vec![
				ReserveAssetDeposited(
					MultiAsset { id: Concrete(T::LocalAssetId::get()), fun: Fungible(0) }.into(),
				),
				BuyExecution {
					fees: MultiAsset { id: Concrete(T::LocalAssetId::get()), fun: Fungible(0) },
					weight_limit: WeightLimit::Unlimited,
				},
				SetAppendix(Xcm(vec![
					RefundSurplus,
					DepositAsset {
						assets: Wild(All),
						max_assets: 1,
						beneficiary: Default::default(),
					},
				])),
			]);
			extend_xcm_for_weight.0.extend(xcm.0.into_iter());
			return extend_xcm_for_weight;
		}
	}
}
