// This file is part of Darwinia.
//
// Copyright (C) 2018-2021 Darwinia Network
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

#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;

// --- core ---
use core::marker::PhantomData;
// --- paritytech ---
use bp_messages::{
	source_chain::{LaneMessageVerifier, Sender},
	LaneId, OutboundLaneData,
};
use bridge_runtime_common::messages::{source::*, *};
use frame_support::{
	traits::{Currency, Imbalance, OnUnbalanced},
	weights::{
		constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	},
};
use pallet_transaction_payment::Multiplier;
use sp_runtime::{FixedPointNumber, Perbill, Perquintill, RuntimeDebug};
// --- darwinia-network ---
use dc_primitives::*;

pub type NegativeImbalance<R> = <pallet_balances::Pallet<R> as Currency<
	<R as frame_system::Config>::AccountId,
>>::NegativeImbalance;

frame_support::parameter_types! {
	/// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
	/// than this will decrease the weight and more will increase.
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	/// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
	/// change the fees more rapidly.
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
	/// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
	/// that combined with `AdjustmentVariable`, we can recover from the minimum.
	/// See `multiplier_can_grow_from_zero`.
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000_u128);
}

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - [0, MAXIMUM_BLOCK_WEIGHT]
///   - [Balance::min, Balance::max]
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;

	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// in `Pangolin Parachain`, extrinsic base weight (smallest non-zero weight) is mapped to
		// 100 MILLI:
		let p = 100 * MILLI_COIN;
		let q = Balance::from(ExtrinsicBaseWeight::get());

		smallvec::smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

/// Logic for the author to get a portion of fees.
pub struct ToStakingPot<R>(PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for ToStakingPot<R>
where
	R: pallet_balances::Config + pallet_collator_selection::Config,
	<R as frame_system::Config>::AccountId: Into<AccountId> + From<AccountId>,
	<R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
	fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
		let staking_pot = <pallet_collator_selection::Pallet<R>>::account_id();

		<pallet_balances::Pallet<R>>::resolve_creating(&staking_pot, amount);
	}
}

pub struct DealWithFees<R>(PhantomData<R>);
impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
where
	R: pallet_balances::Config + pallet_collator_selection::Config,
	<R as frame_system::Config>::AccountId: Into<AccountId> + From<AccountId>,
	<R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
{
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
		if let Some(mut fees) = fees_then_tips.next() {
			if let Some(tips) = fees_then_tips.next() {
				tips.merge_into(&mut fees);
			}
			<ToStakingPot<R> as OnUnbalanced<_>>::on_unbalanced(fees);
		}
	}
}

/// Message verifier that is doing all basic checks.
///
/// This verifier assumes following:
///
/// - all message lanes are equivalent, so all checks are the same;
/// - messages are being dispatched using `pallet-bridge-dispatch` pallet on the target chain.
///
/// Following checks are made:
///
/// - message is rejected if its lane is currently blocked;
/// - message is rejected if there are too many pending (undelivered) messages at the outbound lane;
/// - check that the sender has rights to dispatch the call on target chain using provided dispatch
///   origin;
/// - check that the sender has paid enough funds for both message delivery and dispatch.
#[derive(RuntimeDebug)]
pub struct FromThisChainMessageVerifier<B, R, I>(PhantomData<(B, R, I)>);
impl<B, R, I>
	LaneMessageVerifier<
		AccountIdOf<ThisChain<B>>,
		FromThisChainMessagePayload<B>,
		BalanceOf<ThisChain<B>>,
	> for FromThisChainMessageVerifier<B, R, I>
where
	B: MessageBridge,
	R: pallet_fee_market::Config<I>,
	I: 'static,
	AccountIdOf<ThisChain<B>>: Clone + PartialEq,
	pallet_fee_market::BalanceOf<R, I>: From<BalanceOf<ThisChain<B>>>,
{
	type Error = &'static str;

	fn verify_message(
		submitter: &Sender<AccountIdOf<ThisChain<B>>>,
		delivery_and_dispatch_fee: &BalanceOf<ThisChain<B>>,
		lane: &LaneId,
		lane_outbound_data: &OutboundLaneData,
		payload: &FromThisChainMessagePayload<B>,
	) -> Result<(), Self::Error> {
		// reject message if lane is blocked
		if !ThisChain::<B>::is_outbound_lane_enabled(lane) {
			return Err(OUTBOUND_LANE_DISABLED);
		}

		// reject message if there are too many pending messages at this lane
		let max_pending_messages = ThisChain::<B>::maximal_pending_messages_at_outbound_lane();
		let pending_messages = lane_outbound_data
			.latest_generated_nonce
			.saturating_sub(lane_outbound_data.latest_received_nonce);
		if pending_messages > max_pending_messages {
			return Err(TOO_MANY_PENDING_MESSAGES);
		}

		// Do the dispatch-specific check. We assume that the target chain uses
		// `Dispatch`, so we verify the message accordingly.
		pallet_bridge_dispatch::verify_message_origin(submitter, payload)
			.map_err(|_| BAD_ORIGIN)?;

		// Do the delivery_and_dispatch_fee. We assume that the delivery and dispatch fee always
		// greater than the fee market provided fee.
		if let Some(market_fee) = pallet_fee_market::Pallet::<R, I>::market_fee() {
			let message_fee: pallet_fee_market::BalanceOf<R, I> =
				(*delivery_and_dispatch_fee).into();

			// compare with actual fee paid
			if message_fee < market_fee {
				return Err(TOO_LOW_FEE);
			}
		} else {
			const NO_MARKET_FEE: &str = "The fee market are not ready for accepting messages.";

			return Err(NO_MARKET_FEE);
		}

		Ok(())
	}
}
