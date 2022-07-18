// --- paritytech ---
use frame_support::traits::{EnsureOrigin, Get};
use frame_system::RawOrigin;
// --- darwinia-network ---
use crate::*;
use dp_common_runtime::remote_governance::Config;

pub struct EnsureSpecific;
impl<O: Into<Result<RawOrigin<AccountId>, O>> + From<RawOrigin<AccountId>>> EnsureOrigin<O>
	for EnsureSpecific
{
	type Success = AccountId;

	fn try_origin(o: O) -> Result<Self::Success, O> {
		o.into().and_then(|o| match o {
			RawOrigin::Signed(who)
				if who
					== array_bytes::hex_into_unchecked(
						"0x129d025b24257aabdefac93d00419f06a38e3a5e2314dd6866b16e8f205ce074",
					) =>
				Ok(who),
			r => Err(O::from(r)),
		})
	}
}

pub struct CrabBestFinalized;
impl Get<Hash> for CrabBestFinalized {
	fn get() -> Hash {
		<pallet_bridge_grandpa::BestFinalized<Runtime, WithCrabGrandpa>>::get()
	}
}

frame_support::parameter_types! {
	pub const CheckInterval: BlockNumber = 4 * HOURS;
}

impl Config for Runtime {
	type BridgeAccountIdConverter = bp_crab::AccountIdConverter;
	type BridgeFinalized = CrabBestFinalized;
	type BridgedChainId = CrabChainId;
	type Call = Call;
	type CheckInterval = CheckInterval;
	type EmergencySafeguardOrigin = EnsureSpecific;
	type Event = Event;
}
