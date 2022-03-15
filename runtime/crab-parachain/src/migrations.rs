// --- paritytech ---
use frame_support::{
	traits::{Currency, OnRuntimeUpgrade},
	weights::Weight,
};
// --- darwinia-network ---
use crate::*;

fn accounts() -> Vec<AccountId> {
	[
		// root
		"0x129d025b24257aabdefac93d00419f06a38e3a5e2314dd6866b16e8f205ce074",
		// @Hackfisher
		"0x0a66532a23c418cca12183fee5f6afece770a0bb8725f459d7d1b1b598f91c49",
	]
	.iter()
	.filter_map(|hex_account| {
		if let Ok(account) = array_bytes::hex_try_into(hex_account) {
			Some(account)
		} else {
			None
		}
	})
	.collect()
}

pub struct CustomOnRuntimeUpgrade;
impl OnRuntimeUpgrade for CustomOnRuntimeUpgrade {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		for account in accounts() {
			assert_eq!(Balances::free_balance(account), 100_000 * G_WEI);
		}

		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		for account in accounts() {
			assert_eq!(Balances::free_balance(account), 100_000 * COIN);
		}

		Ok(())
	}

	fn on_runtime_upgrade() -> Weight {
		migrate()
	}
}

fn migrate() -> Weight {
	for account in accounts() {
		Balances::make_free_balance_be(&account, 100_000 * COIN);
	}

	// 0
	RuntimeBlockWeights::get().max_block
}
