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
						"0xc778fc2665f3f6ee9623594e5d1fab9dbd557149542c5edacbcc543a82c9d780",
					) =>
				Ok(who),
			r => Err(O::from(r)),
		})
	}
}

pub struct DarwiniaBestFinalized;
impl Get<Hash> for DarwiniaBestFinalized {
	fn get() -> Hash {
		// <pallet_bridge_grandpa::BestFinalized<Runtime, WithCrabGrandpa>>::get()
		todo!()
	}
}

frame_support::parameter_types! {
	pub const CheckInterval: BlockNumber = DAYS;
}

impl Config for Runtime {
	type BridgeAccountIdConverter = bp_darwinia::AccountIdConverter;
	type BridgeFinalized = DarwiniaBestFinalized;
	type BridgedChainId = DarwiniaChainId;
	type Call = Call;
	type CheckInterval = CheckInterval;
	type EmergencySafeguardOrigin = EnsureSpecific;
	type Event = Event;
}
