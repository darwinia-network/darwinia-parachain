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

//! Prototype module for remote governance.

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

// --- paritytech ---
use bp_runtime::{derive_account_id, ChainId, SourceAccount};
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Get, UnfilteredDispatchable},
	transactional,
	weights::GetDispatchInfo,
};
use frame_system::{ensure_signed, RawOrigin};
use sp_core::H256;
use sp_runtime::traits::{Convert, Dispatchable};

pub use pallet::*;
pub type AccountId<T> = <T as frame_system::Config>::AccountId;

pub trait Rescuer<OuterOrigin> {
	fn allow(user: OuterOrigin) -> bool;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	#[pallet::disable_frame_system_supertrait_check]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The bridge account id converter.
		/// `remote account` + `remote chain id` derive the new account
		type BridgedAccountIdConverter: Convert<H256, Self::AccountId>;

		/// The bridged chain id
		type BridgedChainId: Get<ChainId>;

		/// The rescuer which has the permission to call as root
		type Rescuer: Rescuer<Self::Origin>;

		/// A remote governance call or a rescue call
		type Call: Parameter
			+ Dispatchable<Origin = Self::Origin>
			+ UnfilteredDispatchable<Origin = Self::Origin>
			+ GetDispatchInfo;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Rescuer Call. \[result\]
		RescureCall(DispatchResult),
		/// Governance Call. \[result\]
		GovernanceCall(T::AccountId, DispatchResult),
	}

	#[pallet::error]
	/// Issuing pallet errors.
	pub enum Error<T> {
		/// Origin Must Be Rescuer
		RequireRescuer,
		/// Origin Must Be SourceRoot
		RequireSourceRoot,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Handle relay message sent from the source backing pallet with relay message
		#[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(dispatch_info.weight.saturating_add(10_000), dispatch_info.class)
        })]
		#[transactional]
		pub fn accept_remote_call(
			origin: OriginFor<T>,
			call: Box<<T as Config>::Call>,
		) -> DispatchResultWithPostInfo {
			let user = ensure_signed(origin)?;
			let source_root = Self::derived_source_root();
			ensure!(&source_root == &user, Error::<T>::RequireSourceRoot);

			let res = call.dispatch(RawOrigin::Root.into());
			Self::deposit_event(Event::GovernanceCall(user, res.map(|_| ()).map_err(|e| e.error)));
			Ok(().into())
		}

		#[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(dispatch_info.weight.saturating_add(10_000), dispatch_info.class)
        })]
		#[transactional]
		pub fn rescue_call(
			origin: OriginFor<T>,
			call: Box<<T as Config>::Call>,
		) -> DispatchResultWithPostInfo {
			//let user = ensure_signed(origin)?;
			ensure!(T::Rescuer::allow(origin), Error::<T>::RequireRescuer);

			let res = call.dispatch_bypass_filter(RawOrigin::Root.into());
			Self::deposit_event(Event::RescureCall(res.map(|_| ()).map_err(|e| e.error)));
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn derived_source_root() -> T::AccountId {
			let hex_id =
				derive_account_id::<T::AccountId>(T::BridgedChainId::get(), SourceAccount::Root);
			T::BridgedAccountIdConverter::convert(hex_id)
		}
	}
}
