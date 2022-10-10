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

//! Prototype module for s2s cross chain assets issuing.

pub mod weight;
pub use weight::WeightInfo;

mod evm;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

// --- paritytech ---
use bp_message_dispatch::CallOrigin;
use bp_messages::{
	source_chain::{MessagesBridge, OnDeliveryConfirmed},
	BridgeMessageId, DeliveredMessages, LaneId, MessageNonce,
};
use bp_runtime::{derive_account_id, messages::DispatchFeePayment, ChainId, SourceAccount};
use frame_support::{
	ensure,
	pallet_prelude::*,
	traits::{Currency, ExistenceRequirement, Get, WithdrawReasons},
	weights::PostDispatchInfo,
	PalletId,
};
use frame_system::{ensure_signed, RawOrigin};
use sp_core::H256;
use sp_runtime::{
	traits::{AccountIdConversion, BadOrigin, CheckedDiv, CheckedMul, Convert, Saturating, Zero},
	DispatchErrorWithPostInfo, MultiSignature, MultiSigner, SaturatedConversion,
};
use sp_std::{str, vec, vec::Vec};

pub use pallet::*;
pub type ChainName = Vec<u8>;
pub type AccountId<T> = <T as frame_system::Config>::AccountId;
pub type RingBalance<T> = <<T as Config>::RingCurrency as Currency<AccountId<T>>>::Balance;

/// The parameters box for the pallet runtime call.
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum CallParams<T: Config> {
	#[codec(index = 1)]
    EthereumPalletMessageTransact(Transaction),
}
/// Creating a concrete message payload which would be relay to target chain.
pub trait CreatePayload<
	SourceChainAccountId,
	TargetChainAccountPublic,
	TargetChainSignature,
	T: Config,
>
{
	type Payload: Encode;

	fn encode_call(pallet_index: u8, call_params: CallParams<T>) -> Result<Vec<u8>, &'static str> {
		let mut encoded = vec![pallet_index];
		encoded.append(&mut call_params.encode());
		Ok(encoded)
	}

	fn create(
		origin: CallOrigin<SourceChainAccountId, TargetChainAccountPublic, TargetChainSignature>,
		spec_version: u32,
		weight: u64,
		call_params: CallParams<T>,
		dispatch_fee_payment: DispatchFeePayment,
	) -> Result<Self::Payload, &'static str>;
}

pub trait LatestMessageNoncer {
	fn outbound_latest_generated_nonce(lane_id: LaneId) -> MessageNonce;
    fn inbound_latest_received_nonce(lane_id: LaneId) -> MessageNonce;
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	#[pallet::disable_frame_system_supertrait_check]
	pub trait Config: frame_system::Config {
		/// The pallet id of this pallet
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// The *RING* currency.
		type RingCurrency: Currency<AccountId<Self>>;

		/// The bridge account id converter.
		/// `remote account` + `remote chain id` derive the new account
		type BridgedAccountIdConverter: Convert<H256, Self::AccountId>;

		/// The bridged chain id
		type BridgedChainId: Get<ChainId>;

        /// The bridged smart chain id
        type BridgedSmartChainId: Get<u32>;

		/// Outbound payload creator used for s2s message
		type OutboundPayloadCreator: Parameter
			+ CreatePayload<Self::AccountId, MultiSigner, MultiSignature, Self>;

		/// The lane id of the s2s bridge
		type MessageLaneId: Get<LaneId>;

		type MessagesBridge: MessagesBridge<
			Self::Origin,
			Self::AccountId,
			RingBalance<Self>,
			<<Self as Config>::OutboundPayloadCreator as CreatePayload<
				Self::AccountId,
				MultiSigner,
				MultiSignature,
				Self,
			>>::Payload,
			Error = DispatchErrorWithPostInfo<PostDispatchInfo>,
		>;
		type MessageNoncer: LatestMessageNoncer;
	}

	/// Remote Backing Address, this used to verify the remote caller
	#[pallet::storage]
	#[pallet::getter(fn remote_backing_account)]
	pub type RemoteBackingAccount = StorageValue<_, H160, OptionQuery>;

	/// `(sender, amount)` the user *sender* lock and remote issuing amount of asset
	#[pallet::storage]
	#[pallet::getter(fn transaction_infos)]
	pub type TransactionInfos<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BridgeMessageId,
		(AccountId<T>, RingBalance<T>),
		OptionQuery,
	>;

    pub type ReceivedNonce = StorageValue<
        _,
        Vec<MessageNonce>,
        ValueQuery,
    >;

	/// Period between security limitation. Zero means there is no period limitation.
	#[pallet::storage]
	#[pallet::getter(fn secure_limited_period)]
	pub type SecureLimitedPeriod<T> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn secure_limited_ring_amount)]
	pub type SecureLimitedRingAmount<T> =
		StorageValue<_, (RingBalance<T>, RingBalance<T>), ValueQuery>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			let secure_limited_period = <SecureLimitedPeriod<T>>::get();

			if !secure_limited_period.is_zero() && (now % secure_limited_period).is_zero() {
				<SecureLimitedRingAmount<T>>::mutate(|(used, _)| *used = Zero::zero());

				T::DbWeight::get().reads_writes(2, 1)
			} else {
				T::DbWeight::get().reads(1)
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Handle relay message sent from the source backing pallet with relay message
		#[pallet::weight(
			<T as Config>::WeightInfo::issue_from_remote()
		)]
		pub fn issue_from_remote(
			origin: OriginFor<T>,
			value: RingBalance<T>,
			recipient: AccountId<T>,
            burn_pruned_messages: Vec<MessageNonce>,
            max_lock_pruned_nonce: MessageNonce,
		) -> DispatchResultWithPostInfo {
			let user = ensure_signed(origin)?;

			if let Some(backing) = <RemoteBackingAccount<T>>::get() {
                let backingAccount = T::AccountId::decode(evm::derive_substrate_address(backing)[..])?;
				let target_id = Self::derived_backing_id(backingAccount);
				ensure!(target_id == user, BadOrigin);
			} else {
				return Err(Error::<T>::BackingAccountNone.into());
			}

			// Make sure the total transfer is less than the security limitation
			{
				let (used, limitation) = <SecureLimitedRingAmount<T>>::get();
				ensure!(
					<SecureLimitedPeriod<T>>::get().is_zero()
						|| used.saturating_add(value) <= limitation,
					<Error<T>>::RingDailyLimited
				);
			}

            Self::prun_message(burn_pruned_messages, max_lock_pruned_nonce);

			T::RingCurrency::deposit_creating(&recipient, value);
			Self::deposit_event(Event::TokenIssued(recipient, value));
			Ok(().into())
		}

		#[pallet::weight(
			<T as Config>::WeightInfo::burn_and_remote_unlock()
		)]
		pub fn burn_and_remote_unlock(
			origin: OriginFor<T>,
			spec_version: u32,
			weight: u64,
            gasLimit: U256,
			#[pallet::compact] value: RingBalance<T>,
			#[pallet::compact] fee: RingBalance<T>,
			recipient: H160,
		) -> DispatchResultWithPostInfo {
			let user = ensure_signed(origin)?;

			// Make sure the user's balance is enough to lock
			ensure!(
				T::RingCurrency::free_balance(&user) > value + fee,
				<Error<T>>::InsufficientBalance
			);

			// this pallet account as the submitter of the remote message
			// we need to transfer fee from user to this account to pay the bridge fee
			T::RingCurrency::transfer(
				&user,
				&Self::pallet_account_id(),
				value + fee,
				ExistenceRequirement::KeepAlive,
			)?;

            let remote_unlock_input = evm::ToParachainBacking::encode_unlock_from_remote(recipient, value.into())?;
            let message_id = Self::remote_evm_call(
                spec_version,
                weight,
                fee,
                gasLimit,
                remote_unlock_input,
                )?;
			ensure!(!<TransactionInfos<T>>::contains_key(message_id), Error::<T>::NonceDuplicated);
			<TransactionInfos<T>>::insert(message_id, (user.clone(), value));
			Self::deposit_event(Event::TokenBurnAndRemoteUnlocked(
				T::MessageLaneId::get(),
				message_nonce,
				user,
				recipient,
				value,
			));
			Ok(().into())
		}

        #[pallet::weight(100)]
		pub fn handle_issuing_failure_from_remote(
			origin: OriginFor<T>,
            failure_nonce: MessageNonce,
            burn_pruned_messages: Vec<MessageNonce>,
            max_lock_pruned_nonce: MessageNonce,
		) -> DispatchResultWithPostInfo {
			let user = ensure_signed(origin)?;

			if let Some(backing) = <RemoteBackingAccount<T>>::get() {
                let backingAccount = T::AccountId::decode(evm::derive_substrate_address(backing)[..])?;
				let target_id = Self::derived_backing_id(backingAccount);
				ensure!(target_id == user, BadOrigin);
			} else {
				return Err(Error::<T>::BackingAccountNone.into());
			}

            // verify message
            let failure_message_id: BridgeMessageId = (T::MessageLaneId::get(), failure_nonce);
			ensure!(<TransactionInfos<T>>::contains_key(failure_message_id), Error::<T>::NonceNE);
            <TransactionInfos<T>>::remove(failure_message_id);

            Self::prun_message(burn_pruned_messages, max_lock_pruned_nonce);

			T::RingCurrency::deposit_creating(&recipient, value);
			Self::deposit_event(Event::TokenIssuedForFailure(T::MessageLaneId::get(), failure_nonce, recipient, value));
			Ok(().into())
        }
			
        #[pallet::weight(100)]
        pub fn remote_unlock_failure(
			origin: OriginFor<T>,
			spec_version: u32,
			weight: u64,
            failure_nonce: MessageNonce,
			#[pallet::compact] fee: RingBalance<T>,
        ) {
            // check message
            // 1. message must not be issued
            let received_nonces = <ReceivedNonce<T>>::get();
            ensure!(received_nonces.binary_search(&failure_nonce).is_err(), Error::<T>::MessageAlreadyIssued);
            // 2. message must be delived
            let message_nonce =
                T::MessageNoncer::inbound_latest_received_nonce(T::MessageLaneId::get());
            ensure!(message_nonce >= failure_nonce, Error::<T>::MessageNotDelived);

            // send refund message
            let remote_unlock_for_failure_input = evm::ToParachainBacking::encode_handle_unlock_failure_from_remote(failure_nonce)?;
            let message_id = Self::remote_evm_call(
                spec_version,
                weight,
                fee,
                gasLimit,
                remote_unlock_for_failure_input,
                )?;
			Self::deposit_event(Event::RemoteUnlockForFailure(failure_nonce));
			Ok(().into())
        }

		#[pallet::weight(
			<T as Config>::WeightInfo::set_remote_backing_account()
		)]
		pub fn set_remote_backing_account(
			origin: OriginFor<T>,
			account: H160,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			<RemoteBackingAccount<T>>::put(account.clone());
			Self::deposit_event(Event::RemoteBackingAccountUpdated(account));
			Ok(().into())
		}

		#[pallet::weight(
            <T as Config>::WeightInfo::set_secure_limited_period()
        )]
		pub fn set_secure_limited_period(
			origin: OriginFor<T>,
			period: BlockNumberFor<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			<SecureLimitedPeriod<T>>::put(period);

			Ok(().into())
		}

		#[pallet::weight(
			<T as Config>::WeightInfo::set_security_limitation_ring_amount()
		)]
		pub fn set_security_limitation_ring_amount(
			origin: OriginFor<T>,
			limitation: RingBalance<T>,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			<SecureLimitedRingAmount<T>>::mutate(|(_, limitation_)| *limitation_ = limitation);

			Ok(().into())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// TokenBurnAndRemoteUnlocked \[lane_id, message_nonce, sender, recipient, amount\]
		TokenBurnAndRemoteUnlocked(
			LaneId,
			MessageNonce,
			AccountId<T>,
			AccountId<T>,
			RingBalance<T>,
		),
		/// [recipient, amount]
		TokenIssued(AccountId<T>, RingBalance<T>),
		/// Update remote backing address \[account\]
		RemoteBackingAccountUpdated(H160),
        /// issue for failure unlock [lane_id, failure_nonce, recipient, amount]
		TokenIssuedForFailure(LaneId, MessageNonce, AccountId<T>, RingBalance<T>),
        /// request remote unlock for failure issue [[failure_nonce]
        RemoteUnlockForFailure(MessageNonce);
	}

	#[pallet::error]
	/// Issuing pallet errors.
	pub enum Error<T> {
		/// Redeem Daily Limited
		RingDailyLimited,
		/// Insufficient balance.
		InsufficientBalance,
		/// Message nonce duplicated.
		NonceDuplicated,
		/// Backing not configured
		BackingAccountNone,
        /// Message nonce is not exist
        NonceNE,
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub secure_limited_period: BlockNumberFor<T>,
		pub secure_limited_ring_amount: RingBalance<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self { secure_limited_period: Zero::zero(), secure_limited_ring_amount: Zero::zero() }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			<SecureLimitedPeriod<T>>::put(self.secure_limited_period);
			<SecureLimitedRingAmount<T>>::put((
				<RingBalance<T>>::zero(),
				self.secure_limited_ring_amount,
			));
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn pallet_account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub fn derived_backing_id(backing_account: T::AccountId) -> T::AccountId {
			let hex_id = derive_account_id::<T::AccountId>(
				T::BridgedChainId::get(),
				SourceAccount::Account(backing_account),
			);
			T::BridgedAccountIdConverter::convert(hex_id)
		}

        pub fn prun_message(
            pruned_messages: Vec<MessageNonce>,
            max_lock_pruned_nonce: MessageNonce
        ) {
            <ReceivedNonce<T>>::mutate(|received| {
                received.retain(|&r| r > max_lock_pruned_nonce);
                let message_nonce =
                    T::MessageNoncer::inbound_latest_received_nonce(T::MessageLaneId::get());
                received.push(message_nonce);
            });
            for nonce in pruned_messages {
                let message_id: BridgeMessageId = (T::MessageLaneId::get(), nonce);
                if (<TransactionInfos<T>>::contains_key(message_id)) {
                    <TransactionInfos<T>>::remove(message_id);
                }
            }
        }

        pub fn remote_evm_call(
            spec_version: u32,
            weight: u64,
			fee: RingBalance<T>,
            gasLimit: U256,
            input: Vec<u8>,
        ) -> Result<BridgeMessageId, DispatchError> {
			if let Some(backing) = <RemoteBackingAccount<T>>::get() {
                let ethereum_transaction = evm::new_ethereum_transaction(
                    T::BridgedSmartChainId::get(),
                    backing,
                    gasLimit,
                    input)?;
                let payload = T::OutboundPayloadCreator::create(
                    CallOrigin::SourceAccount(Self::pallet_account_id()),
                    spec_version,
                    weight,
                    CallParams::EthereumPalletMessageTransact(ethereum_transaction),
                    DispatchFeePayment::AtSourceChain,
                    )?;
                T::MessagesBridge::send_message(
                    RawOrigin::Signed(Self::pallet_account_id()).into(),
                    T::MessageLaneId::get(),
                    payload,
                    fee,
                    )?;

                let message_nonce =
                    T::MessageNoncer::outbound_latest_generated_nonce(T::MessageLaneId::get());
                let message_id: BridgeMessageId = (T::MessageLaneId::get(), message_nonce);
                return Ok(message_id);
            } else {
				return Err(Error::<T>::BackingAccountNone.into());
            }
        }
	}
}
