// --- paritytech ---
use pallet_multisig::Config;
// --- darwinia-network ---
use crate::{weights::pallet_multisig::WeightInfo, *};

frame_support::parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = dc_primitives::crab_deposit(1, 88);
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = dc_primitives::crab_deposit(0, 32);
	pub const MaxSignatories: u16 = 100;
}

impl Config for Runtime {
	type Currency = Ring;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = WeightInfo<Runtime>;
}
