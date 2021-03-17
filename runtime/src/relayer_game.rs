// --- darwinia ---
pub use darwinia_relayer_game::Instance0 as EthereumRelayerGameInstance;

// --- substrate ---
use frame_support::traits::LockIdentifier;
// --- darwinia ---
use crate::*;
use darwinia_relay_primitives::relayer_game::*;
use darwinia_relayer_game::Config;
use ethereum_primitives::EthereumBlockNumber;

pub struct EthereumRelayerGameAdjustor;
impl AdjustableRelayerGame for EthereumRelayerGameAdjustor {
	type Moment = BlockNumber;
	type Balance = Balance;
	type RelayHeaderId = EthereumBlockNumber;

	fn max_active_games() -> u8 {
		32
	}

	fn affirm_time(round: u32) -> Self::Moment {
		match round {
			// 1.5 mins
			0 => 15,
			// 0.5 mins
			_ => 5,
		}
	}

	fn complete_proofs_time(round: u32) -> Self::Moment {
		match round {
			// 1.5 mins
			0 => 15,
			// 0.5 mins
			_ => 5,
		}
	}

	fn update_sample_points(sample_points: &mut Vec<Vec<Self::RelayHeaderId>>) {
		sample_points.push(vec![sample_points.last().unwrap().last().unwrap() - 1]);
	}

	fn estimate_stake(round: u32, affirmations_count: u32) -> Self::Balance {
		match round {
			0 => match affirmations_count {
				0 => 1000 * COIN,
				_ => 1500 * COIN,
			},
			_ => 100 * COIN,
		}
	}
}
frame_support::parameter_types! {
	pub const EthereumRelayerGameLockId: LockIdentifier = *b"ethrgame";
}
impl Config<EthereumRelayerGameInstance> for Runtime {
	type RingCurrency = Ring;
	type LockId = EthereumRelayerGameLockId;
	type RingSlash = Treasury;
	type RelayerGameAdjustor = EthereumRelayerGameAdjustor;
	type RelayableChain = EthereumRelay;
	type WeightInfo = ();
}
