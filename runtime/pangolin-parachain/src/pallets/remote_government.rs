// --- paritytech ---
use frame_support::traits::EnsureOrigin;
use frame_system::RawOrigin;
// --- darwinia-network ---
use crate::*;
use bp_pangolin::AccountIdConverter;
use dp_common_runtime::remote_government::Config;

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
						"0x72819fbc1b93196fa230243947c1726cbea7e33044c7eb6f736ff345561f9e4c",
					) =>
				Ok(who),
			r => Err(O::from(r)),
		})
	}
}

impl Config for Runtime {
	type BridgeAccountIdConverter = AccountIdConverter;
	type BridgedChainId = PangolinChainId;
	type Call = Call;
	type EmergencySafeguardOrigin = EnsureSpecific;
	type Event = Event;
}
