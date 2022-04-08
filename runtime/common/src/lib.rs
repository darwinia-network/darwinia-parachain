// --- core ---
use core::marker::PhantomData;
// --- paritytech ---
use frame_support::{
	traits::{Currency, Imbalance, OnUnbalanced},
	weights::{
		constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	},
};
use pallet_transaction_payment::Multiplier;
use sp_runtime::{FixedPointNumber, Perbill, Perquintill};
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
		// in `Pangolin Parachain`, extrinsic base weight (smallest non-zero weight) is mapped to 100 MILLI:
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
