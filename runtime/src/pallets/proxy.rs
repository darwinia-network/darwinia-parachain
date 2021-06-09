// --- crates.io ---
use codec::{Decode, Encode};
// --- substrate ---
use frame_support::traits::InstanceFilter;
use pallet_proxy::{weights::SubstrateWeight, Config};
use sp_runtime::{traits::BlakeTwo256, RuntimeDebug};
// --- darwinia ---
use crate::*;

/// The type used to represent the kinds of proxying allowed.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Encode, Decode, RuntimeDebug)]
pub enum ProxyType {
	Any,
	NonTransfer,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
impl InstanceFilter<Call> for ProxyType {
	fn filter(&self, c: &Call) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => matches!(
				c,
				Call::System(..)
					| Call::Timestamp(..)
					| Call::Sudo(..) | Call::Utility(..)
					| Call::Proxy(..) | Call::Multisig(..)
			),
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			(ProxyType::NonTransfer, _) => true,
		}
	}
}
frame_support::parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = constants::deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = constants::deposit(0, 33);
	pub const MaxProxies: u16 = 32;
	pub const AnnouncementDepositBase: Balance = constants::deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = constants::deposit(0, 66);
	pub const MaxPending: u16 = 32;
}
impl Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Ring;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = MaxProxies;
	type MaxPending = MaxPending;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
	type WeightInfo = SubstrateWeight<Runtime>;
}
