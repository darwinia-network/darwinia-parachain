// --- paritytech ---
use frame_support::weights::ConstantMultiplier;
use pallet_transaction_payment::{Config, CurrencyAdapter, TargetedFeeAdjustment};
// --- darwinia-network ---
use crate::*;

pub type SlowAdjustingFeeUpdate<R> =
	TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;

frame_support::parameter_types! {
	pub const TransactionByteFee: Balance = 5 * MILLI_COIN;
	pub const OperationalFeeMultiplier: u8 = 5;
}

impl Config for Runtime {
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
	type LengthToFee = ConstantMultiplier<Balance, TransactionByteFee>;
	type OnChargeTransaction = CurrencyAdapter<Ring, DealWithFees<Runtime>>;
	type OperationalFeeMultiplier = OperationalFeeMultiplier;
	type WeightToFee = WeightToFee;
}
