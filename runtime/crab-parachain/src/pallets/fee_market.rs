pub use pallet_fee_market::Instance1 as WithCrabFeeMarket;

// --- core ---
use core::cmp;
// --- substrate ---
use frame_support::{traits::LockIdentifier, PalletId};
use sp_runtime::{traits::UniqueSaturatedInto, Permill};
// --- darwinia ---
use crate::*;
use pallet_fee_market::{BalanceOf, Config, Slasher};

pub struct FeeMarketSlasher;
impl<T: Config<I>, I: 'static> Slasher<T, I> for FeeMarketSlasher {
	fn cal_slash_amount(
		locked_collateral: BalanceOf<T, I>,
		timeout: T::BlockNumber,
	) -> BalanceOf<T, I> {
		let slash_each_block = 2 * COIN;
		let slash_value = UniqueSaturatedInto::<Balance>::unique_saturated_into(timeout)
			.saturating_mul(UniqueSaturatedInto::<Balance>::unique_saturated_into(slash_each_block))
			.unique_saturated_into();

		cmp::min(locked_collateral, slash_value)
	}
}

frame_support::parameter_types! {
	pub const TreasuryPalletId: PalletId = PalletId(*b"da/trsry");
	pub const FeeMarketLockId: LockIdentifier = *b"da/feecr";

	pub const MinimumRelayFee: Balance = 15 * COIN;
	pub const CollateralPerOrder: Balance = 50 * COIN;
	pub const Slot: BlockNumber = 600;

	pub const DutyRelayersRewardRatio: Permill = Permill::from_percent(60);
	pub const MessageRelayersRewardRatio: Permill = Permill::from_percent(80);
	pub const ConfirmRelayersRewardRatio: Permill = Permill::from_percent(20);
	pub const AssignedRelayerSlashRatio: Permill = Permill::from_percent(20);
}

impl Config<WithCrabFeeMarket> for Runtime {
	type AssignedRelayerSlashRatio = AssignedRelayerSlashRatio;
	type CollateralPerOrder = CollateralPerOrder;
	type ConfirmRelayersRewardRatio = ConfirmRelayersRewardRatio;
	type Currency = Ring;
	type DutyRelayersRewardRatio = DutyRelayersRewardRatio;
	type Event = Event;
	type LockId = FeeMarketLockId;
	type MessageRelayersRewardRatio = MessageRelayersRewardRatio;
	type MinimumRelayFee = MinimumRelayFee;
	type Slasher = FeeMarketSlasher;
	type Slot = Slot;
	type TreasuryPalletId = TreasuryPalletId;
	type WeightInfo = ();
}
