// --- substrate ---
use sp_runtime::{DispatchResult, ModuleId};
// --- darwinia ---
use crate::*;
use darwinia_ethereum_backing::Config;
use darwinia_support::traits::OnDepositRedeem;

pub struct IgnoreStaking;
impl OnDepositRedeem<AccountId, Balance> for IgnoreStaking {
	fn on_deposit_redeem(
		_: &AccountId,
		_: &AccountId,
		_: Balance,
		_: u64,
		_: u8,
	) -> DispatchResult {
		Err("Ignore Staking")?
	}
}

frame_support::parameter_types! {
	pub const EthereumBackingModuleId: ModuleId = ModuleId(*b"da/ethbk");
	pub const EthereumBackingFeeModuleId: ModuleId = ModuleId(*b"da/ethfe");
	pub const RingLockLimit: Balance = 10_000_000 * COIN;
	pub const KtonLockLimit: Balance = 1000 * COIN;
	pub const AdvancedFee: Balance = 50 * COIN;
	pub const SyncReward: Balance = 1000 * COIN;
}
impl Config for Runtime {
	type ModuleId = EthereumBackingModuleId;
	type FeeModuleId = EthereumBackingFeeModuleId;
	type Event = Event;
	type RedeemAccountId = AccountId;
	type EthereumRelay = EthereumRelay;
	type OnDepositRedeem = IgnoreStaking;
	type RingCurrency = Ring;
	type KtonCurrency = Kton;
	type RingLockLimit = RingLockLimit;
	type KtonLockLimit = KtonLockLimit;
	type AdvancedFee = AdvancedFee;
	type SyncReward = SyncReward;
	type EcdsaAuthorities = EthereumRelayAuthorities;
	type WeightInfo = ();
}
