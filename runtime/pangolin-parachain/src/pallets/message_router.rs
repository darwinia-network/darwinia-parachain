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
	pub MoonbeamUnitWeightCost: Weight = 100_000_000;
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
		X1(Parachain(1000))
	);
}

impl Config for Runtime {
	type Event = Event;
	type AssetModifierOrigin = EnsureRoot<AccountId>;
	type MoonbeamWeigher = FixedWeightBounds<MoonbeamUnitWeightCost, Call, MoonbeamMaxInstructions>;
	type LocalWeigher = FixedWeightBounds<UnitWeightCost, Call, MaxInstructions>;
	type LocalAssetId = AnchoringSelfReserve;
	type LocationInverter = LocationInverter<Ancestry>;
	type SelfLocationInSibl = SelfLocationInSibl;
	type AssetTransactor = LocalAssetTransactor;
	type MoonbeamLocation = MoonbeamLocation;
	type ExecuteXcmOrigin = EnsureXcmOrigin<Origin, LocalOriginToLocation>;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmSender = XcmRouter;
}
