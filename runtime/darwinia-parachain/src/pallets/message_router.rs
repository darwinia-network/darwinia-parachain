// --- paritytech ---
use frame_support::{traits::PalletInfoAccess, weights::Weight};
use frame_system::EnsureRoot;
use xcm::prelude::*;
use xcm_builder::{EnsureXcmOrigin, FixedWeightBounds, LocationInverter};
use xcm_executor::XcmExecutor;
// --- darwinia-network ---
use crate::*;
use dp_common_runtime::message_router::Config;

frame_support::parameter_types! {
	pub const MoonbeamMaxInstructions: u32 = 100;
	// https://github.com/PureStake/moonbeam/blob/master/runtime/moonbeam/src/xcm_config.rs#L201
	pub MoonbeamUnitWeightCost: Weight = 200_000_000;
	pub SelfLocationInSibl: MultiLocation = MultiLocation::new(
		1,
		X1(Parachain(ParachainInfo::parachain_id().into()))
	);
	pub AnchoringSelfReserve: MultiLocation = MultiLocation::new(
		0,
		X1(PalletInstance(<Balances as PalletInfoAccess>::index() as u8))
	);
	pub MoonbeamLocation: MultiLocation = MultiLocation::new(
		1,
		X1(Parachain(2004))
	);
}

impl Config for Runtime {
	type ConfigModifierOrigin = EnsureRoot<AccountId>;
	type Event = Event;
	type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type LocalAssetId = AnchoringSelfReserve;
	type LocalWeigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type LocationInverter = LocationInverter<Ancestry>;
	type MoonbeamLocation = MoonbeamLocation;
	type MoonbeamWeigher = FixedWeightBounds<MoonbeamUnitWeightCost, Call, MoonbeamMaxInstructions>;
	type SelfLocationInSibl = SelfLocationInSibl;
	type WeightInfo = ();
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmSender = XcmRouter;
}
