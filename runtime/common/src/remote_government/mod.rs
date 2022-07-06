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
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Get, UnfilteredDispatchable},
	weights::GetDispatchInfo,
};
use frame_system::{ensure_signed, RawOrigin};
use sp_core::H256;
use sp_runtime::traits::{Convert, Zero};
use sp_std::boxed::Box;
// --- darwinia-network ---
use bp_runtime::{derive_account_id, ChainId, SourceAccount};

type AnyCall<T> = Box<<T as Config>::Call>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Remote government call or a emergency call.
		type Call: Parameter + GetDispatchInfo + UnfilteredDispatchable<Origin = Self::Origin>;

		/// Origin from which the root call can be made under the emergency mode.
		type EmergencySafeguardOrigin: EnsureOrigin<Self::Origin>;

		type CheckInterval: Get<Self::BlockNumber>;

		type BridgeFinalized: Get<H256>;

		/// The bridged chain id.
		type BridgedChainId: Get<ChainId>;

		/// The bridge account id converter.
		/// `remote account` + `remote chain id` derive the new account.
		type BridgeAccountIdConverter: Convert<H256, Self::AccountId>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Bridge's GRANDPA finality has stalled for a long time, enter the emergency mode.
		Emergency,
		/// Recover from the emergency mode.
		Recovery,
		/// Remote call just enacted. \[result\]
		RemoteCallEnacted { result: DispatchResult },
		/// Emergency safeguard just took place. \[result\]
		EmergencySafeguardDone { result: DispatchResult },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Origin MUST be `SourceRoot`.
		RequireSourceRoot,
		/// Only available on emergency mode.
		EmergencyOnly,
	}

	#[pallet::storage]
	#[pallet::getter(fn previous_bridge_finalized)]
	pub type PreviousBridgeFinalized<T> = StorageValue<_, H256, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn emergency)]
	pub type Emergency<T> = StorageValue<_, bool, ValueQuery, DefaultForEmergency>;
	#[pallet::type_value]
	pub fn DefaultForEmergency() -> bool {
		false
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			// If emergency, we check the sync state each block.
			if Self::emergency() {
				if Self::try_sync_latest_bridge_finalized() {
					<Emergency<T>>::kill();

					Self::deposit_event(Event::Recovery);

					T::DbWeight::get().reads_writes(2, 2)
				} else {
					T::DbWeight::get().reads(2)
				}
			} else if (now % T::CheckInterval::get()).is_zero() {
				if Self::try_sync_latest_bridge_finalized() {
					T::DbWeight::get().reads_writes(2, 1)
				} else {
					<Emergency<T>>::put(true);

					Self::deposit_event(Event::Emergency);

					T::DbWeight::get().reads_writes(2, 2)
				}
			} else {
				T::DbWeight::get().reads(1)
			}
		}
	}
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(dispatch_info.weight.saturating_add(10_000), dispatch_info.class)
        })]
		pub fn emergency_safeguard(
			origin: OriginFor<T>,
			call: AnyCall<T>,
		) -> DispatchResultWithPostInfo {
			if !Self::emergency() {
				Err(<Error<T>>::EmergencyOnly)?;
			}

			T::EmergencySafeguardOrigin::ensure_origin(origin)?;

			let res = call.dispatch_bypass_filter(RawOrigin::Root.into());

			Self::deposit_event(Event::EmergencySafeguardDone {
				result: res.map(|_| ()).map_err(|e| e.error),
			});

			// Sudo user does not pay a fee.
			Ok(Pays::No.into())
		}

		/// Handle relay message sent from the source backing pallet with relay message
		#[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(dispatch_info.weight.saturating_add(10_000), dispatch_info.class)
        })]
		pub fn enact_remote_call(
			origin: OriginFor<T>,
			call: AnyCall<T>,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;
			let source_root = Self::derived_source_root();

			ensure!(source_root == sender, Error::<T>::RequireSourceRoot);

			let res = call.dispatch_bypass_filter(RawOrigin::Root.into());

			Self::deposit_event(Event::RemoteCallEnacted {
				result: res.map(|_| ()).map_err(|e| e.error),
			});

			// Sudo user does not pay a fee.
			Ok(Pays::No.into())
		}
	}
	impl<T: Config> Pallet<T> {
		fn try_sync_latest_bridge_finalized() -> bool {
			<PreviousBridgeFinalized<T>>::try_mutate(|previous_bridge_finalized| {
				let best_finalized = T::BridgeFinalized::get();

				if *previous_bridge_finalized == best_finalized {
					Err(())
				} else {
					*previous_bridge_finalized = best_finalized;

					Ok(())
				}
			})
			.is_ok()
		}

		pub(crate) fn derived_source_root() -> T::AccountId {
			let hex_id =
				derive_account_id::<T::AccountId>(T::BridgedChainId::get(), SourceAccount::Root);

			T::BridgeAccountIdConverter::convert(hex_id)
		}
	}
}
pub use pallet::*;
