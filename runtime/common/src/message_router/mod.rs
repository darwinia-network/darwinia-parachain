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

pub mod barriers;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod location_conversion;
mod weights;

// --- paritytech ---
use frame_support::{pallet_prelude::*, traits::Get};
use xcm::prelude::*;
use xcm_executor::traits::WeightBounds;

pub type AssetUnitsPerSecond = u128;

/// router target
#[derive(Clone, Encode, Decode, TypeInfo, PartialEq, Debug)]
pub enum Target {
	Moonbeam,
	Astar,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::message_router::weights::WeightInfo;
	use frame_support::{log, weights::constants::WEIGHT_PER_SECOND};
	use frame_system::pallet_prelude::*;
	use sp_std::{boxed::Box, vec};
	use xcm_executor::traits::InvertLocation;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type ConfigModifierOrigin: EnsureOrigin<Self::Origin>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// Required origin for executing XCM messages.
		type ExecuteXcmOrigin: EnsureOrigin<
			<Self as frame_system::Config>::Origin,
			Success = MultiLocation,
		>;
		type LocalAssetId: Get<MultiLocation>;
		/// Used to calculate the weight required for the local execution of xcm.
		type LocalWeigher: WeightBounds<Self::Call>;
		/// Means of inverting a location.
		type LocationInverter: InvertLocation;
		type MoonbeamLocation: Get<MultiLocation>;
		/// Used to calculate the weight required for the moonbeam execution of xcm.
		type MoonbeamWeigher: WeightBounds<Self::Call>;
		type AstarLocation: Get<MultiLocation>;
		/// Used to calculate the weight required for the moonbeam execution of xcm.
		type AstarWeigher: WeightBounds<Self::Call>;
		/// This chain location relative to sibling chain
		type SelfLocationInSibl: Get<MultiLocation>;
		type WeightInfo: WeightInfo;
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
		/// Deposited when successfully routed.
		/// (send origin, route target, remote xcm, required weight, tokens used)
		ForwardTo(MultiLocation, Target, Xcm<()>, Weight, u128),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Not config the target paraId's info for XCM execution.
		TargetXcmExecNotConfig,
		/// The message's weight could not be determined.
		UnweighableMessage,
		/// Failed to transfer xcm fee.
		FailedPayXcmFee,
		BadVersion,
		/// MultiLocation value too large to descend further.
		MultiLocationFull,
		/// Failed to send xcm.
		XcmSendFailed,
		/// Failed to convert account id to [u8; 32].
		AccountIdConversionFailed,
	}

	/// Stores the units per second executed by the target chain for local asset(e.g. CRAB).
	/// This is used to know how to pay for XCM remote execution use local asset.
	/// For example:
	/// key: {parents: 1, Parachain(2023)}, val: 14719736222326895902025
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
		/// Update the units per second of local asset used in target chain.
		#[pallet::weight(
			<T as Config>::WeightInfo::set_target_xcm_exec_config()
		)]
		pub fn set_target_xcm_exec_config(
			origin: OriginFor<T>,
			target_location: MultiLocation,
			local_asset_units_per_second: AssetUnitsPerSecond,
		) -> DispatchResultWithPostInfo {
			T::ConfigModifierOrigin::ensure_origin(origin)?;

			TargetXcmExecConfig::<T>::insert(&target_location, &local_asset_units_per_second);

			Self::deposit_event(Event::TargetXcmExecConfigChanged {
				target_location,
				local_asset_units_per_second,
			});

			Ok(().into())
		}

		/// Deliver received LCMP messages to other parachains.
		/// 1. Calculate the fee for xcm remote execution
		/// 2. Transfer xcm fee to target sovereign account
		/// 3. Assemble xcm that needs to be executed remotely
		/// 4. Send xcm to target chain
		#[pallet::weight(
			<T as Config>::WeightInfo::forward()
		)]
		pub fn forward(
			origin: OriginFor<T>,
			target: Target,
			message: Box<VersionedXcm<<T as frame_system::Config>::Call>>,
		) -> DispatchResultWithPostInfo {
			// MultiLocation origin used to execute xcm
			let origin_location = T::ExecuteXcmOrigin::ensure_origin(origin.clone())?;
			let account_id = ensure_signed(origin)?;
			// U8 account used in DescendOrigin instruction
			let raw_account = <[u8; 32]>::try_from(account_id.encode())
				.map_err(|_| Error::<T>::AccountIdConversionFailed)?;

			let mut remote_xcm: Xcm<<T as frame_system::Config>::Call> =
				(*message).try_into().map_err(|()| Error::<T>::BadVersion)?;

			// Calculate the execution fee required for remote xcm execution
			// fee = fee_per_second * (weight/weight_per_second)
			let local_asset_units_per_second: AssetUnitsPerSecond;
			let remote_weight: Weight;
			match target {
				Target::Moonbeam => {
					local_asset_units_per_second =
						TargetXcmExecConfig::<T>::get(T::MoonbeamLocation::get())
							.ok_or(Error::<T>::TargetXcmExecNotConfig)?;
					remote_weight = T::MoonbeamWeigher::weight(&mut Self::extend_remote_xcm(
						raw_account,
						remote_xcm.clone(),
						MultiAsset { id: AssetId::from(T::LocalAssetId::get()), fun: Fungible(0) },
					))
					.map_err(|()| Error::<T>::UnweighableMessage)?;
				},
				Target::Astar => {
					local_asset_units_per_second =
						TargetXcmExecConfig::<T>::get(T::AstarLocation::get())
							.ok_or(Error::<T>::TargetXcmExecNotConfig)?;
					remote_weight = T::AstarWeigher::weight(&mut Self::extend_remote_xcm(
						raw_account,
						remote_xcm.clone(),
						MultiAsset { id: AssetId::from(T::LocalAssetId::get()), fun: Fungible(0) },
					))
					.map_err(|()| Error::<T>::UnweighableMessage)?;
				},
			}
			let amount = local_asset_units_per_second.saturating_mul(remote_weight as u128)
				/ (WEIGHT_PER_SECOND as u128);
			let remote_xcm_fee =
				MultiAsset { id: AssetId::from(T::LocalAssetId::get()), fun: Fungible(amount) };

			// Transfer xcm execution fee to target sovereign account
			let mut local_xcm;
			match target {
				Target::Moonbeam => {
					local_xcm = Xcm(vec![TransferAsset {
						assets: remote_xcm_fee.clone().into(),
						beneficiary: T::MoonbeamLocation::get(),
					}]);
				},
				Target::Astar => {
					local_xcm = Xcm(vec![TransferAsset {
						assets: remote_xcm_fee.clone().into(),
						beneficiary: T::AstarLocation::get(),
					}]);
				},
			}
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
				log::error!("Failed transfer xcm fee with {:?}", error);
				Error::<T>::FailedPayXcmFee
			})?;

			// Toggle the xcm_fee relative to a target context
			let ancestry = T::LocationInverter::ancestry();
			let mut remote_xcm_fee_anchor_dest = remote_xcm_fee;
			match target {
				Target::Moonbeam => {
					remote_xcm_fee_anchor_dest
						.reanchor(&T::MoonbeamLocation::get(), &ancestry)
						.map_err(|()| Error::<T>::MultiLocationFull)?;
					remote_xcm = Self::extend_remote_xcm(
						raw_account,
						remote_xcm,
						remote_xcm_fee_anchor_dest,
					);
					// Send remote xcm to target
					T::XcmSender::send_xcm(T::MoonbeamLocation::get(), remote_xcm.clone().into())
						.map_err(|_| Error::<T>::XcmSendFailed)?;
				},
				Target::Astar => {
					remote_xcm_fee_anchor_dest
						.reanchor(&T::AstarLocation::get(), &ancestry)
						.map_err(|()| Error::<T>::MultiLocationFull)?;
					remote_xcm = Self::extend_remote_xcm(
						raw_account,
						remote_xcm,
						remote_xcm_fee_anchor_dest,
					);
					// Send remote xcm to target
					T::XcmSender::send_xcm(T::AstarLocation::get(), remote_xcm.clone().into())
						.map_err(|_| Error::<T>::XcmSendFailed)?;
				},
			}

			Self::deposit_event(Event::ForwardTo(
				origin_location,
				target,
				remote_xcm.into(),
				remote_weight,
				amount,
			));
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Extend xcm for remote execution
		fn extend_remote_xcm(
			raw_account: [u8; 32],
			xcm: Xcm<<T as frame_system::Config>::Call>,
			fee: MultiAsset,
		) -> Xcm<<T as frame_system::Config>::Call> {
			let mut extend_xcm = Xcm(vec![
				ReserveAssetDeposited(fee.clone().into()),
				BuyExecution { fees: fee, weight_limit: WeightLimit::Unlimited },
				// Deposit surplus tokens back into our sovereign account
				SetAppendix(Xcm(vec![
					RefundSurplus,
					DepositAsset {
						assets: Wild(All),
						max_assets: 1,
						beneficiary: T::SelfLocationInSibl::get(),
					},
				])),
				DescendOrigin(X1(AccountId32 { network: NetworkId::Any, id: raw_account })),
			]);
			extend_xcm.0.extend(xcm.0.into_iter());

			extend_xcm
		}
	}
}

pub use pallet::*;
