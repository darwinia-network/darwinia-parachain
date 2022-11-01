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
					== array_bytes::hex_n_into_unchecked(
						"0x72819fbc1b93196fa230243947c1726cbea7e33044c7eb6f736ff345561f9e4c",
					) =>
				Ok(who),
			r => Err(O::from(r)),
		})
	}
}

pub struct PangolinBestFinalized;
impl Get<Hash> for PangolinBestFinalized {
	fn get() -> Hash {
		<pallet_bridge_grandpa::BestFinalized<Runtime, WithPangolinGrandpa>>::get()
	}
}

frame_support::parameter_types! {
	pub const CheckInterval: BlockNumber = 2 * HOURS;
}

impl Config for Runtime {
	type BridgeAccountIdConverter = bp_pangolin::AccountIdConverter;
	type BridgeFinalized = PangolinBestFinalized;
	type BridgedChainId = PangolinChainId;
	type CheckInterval = CheckInterval;
	type EmergencySafeguardOrigin = EnsureSpecific;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
}
