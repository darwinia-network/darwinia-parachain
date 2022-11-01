// --- paritytech ---
use cumulus_pallet_parachain_system::ParachainSetCode;
use frame_support::{
	dispatch::DispatchClass,
	traits::{ConstU32, Everything},
	weights::{constants::WEIGHT_PER_SECOND, Weight},
};
use frame_system::{
	limits::{BlockLength, BlockWeights},
	Config,
};
use pallet_balances::AccountData;
use sp_runtime::{
	traits::{AccountIdLookup, BlakeTwo256},
	Perbill,
};
use sp_version::RuntimeVersion;
// --- darwinia-network ---
use crate::{
	weights::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight},
	*,
};

/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
/// Operational  extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

/// We allow for .5 seconds of compute with a 12 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND.set_proof_size(1_000).saturating_div(2);

frame_support::parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub BlockWeights: limits::BlockWeights =
		limits::BlockWeights::with_sensible_defaults(MAXIMUM_BLOCK_WEIGHT, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 18;
}

impl Config for Runtime {
	type AccountData = AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = Everything;
	type BlockHashCount = BlockHashCount;
	type BlockLength = RuntimeBlockLength;
	type BlockNumber = BlockNumber;
	type BlockWeights = RuntimeBlockWeights;
	type DbWeight = RocksDbWeight;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Index = Nonce;
	type Lookup = AccountIdLookup<AccountId, ()>;
	type MaxConsumers = ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ParachainSetCode<Self>;
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = SS58Prefix;
	type SystemWeightInfo = ();
	type Version = Version;
}
