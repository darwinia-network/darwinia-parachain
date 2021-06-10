// --- parity ---
use pallet_transaction_payment::{Config, CurrencyAdapter, TargetedFeeAdjustment};
// --- darwinia ---
use crate::*;

pub type SlowAdjustingFeeUpdate<R> =
	TargetedFeeAdjustment<R, TargetBlockFullness, AdjustmentVariable, MinimumMultiplier>;

frame_support::parameter_types! {
	pub const TransactionByteFee: Balance = 5 * MILLI;
}

impl Config for Runtime {
	type OnChargeTransaction = CurrencyAdapter<Ring, DealWithFees<Runtime>>;
	type TransactionByteFee = TransactionByteFee;
	type WeightToFee = WeightToFee;
	type FeeMultiplierUpdate = SlowAdjustingFeeUpdate<Self>;
}
