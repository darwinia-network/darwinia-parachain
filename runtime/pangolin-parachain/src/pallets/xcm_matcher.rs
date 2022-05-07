use crate::SelfLocation;
use frame_support::traits::Get;
use sp_runtime::traits::CheckedConversion;
use sp_std::{convert::TryFrom, marker::PhantomData};
use xcm::latest::{
	AssetId::{Concrete},
	Fungibility::Fungible,
	MultiAsset, MultiLocation,
};
use xcm_executor::traits::MatchesFungible;

/// Converts a `MultiAsset` into balance `B` if it is a concrete fungible with an id equal to that
/// given by `T`'s `Get` or `SelfLocation`'s `Get`.
///
/// # Example
///
/// ```
/// use xcm::latest::{MultiLocation, Parent};
/// use xcm::prelude::{Parachain, X1};
/// use xcm_builder::IsConcrete;
/// use xcm_executor::traits::MatchesFungible;
/// use pangolin_parachain_runtime::ParachainInfo;
/// frame_support::parameter_types! {
/// 	pub TargetLocation: MultiLocation = Parent.into();
/// }
///
/// # fn test_target() {
/// let asset = (Parent, 999).into();
/// // match `asset` if it is a concrete asset in `TargetLocation`.
/// assert_eq!(<IsConcrete<TargetLocation> as MatchesFungible<u128>>::matches_fungible(&asset), Some(999));
/// # }
///
/// # fn test_self() {
/// let asset_xcm = (MultiLocation::new(1, X1(Parachain(ParachainInfo::parachain_id().into()))), 999).into();
/// // match `asset` if it is a concrete asset in `SelfLocation`.
/// assert_eq!(<IsConcrete<TargetLocation> as MatchesFungible<u128>>::matches_fungible(&asset_xcm), Some(999));
/// # }
/// ```
pub struct IsPangolinConcrete<T>(PhantomData<T>);
impl<T: Get<MultiLocation>, B: TryFrom<u128>> MatchesFungible<B> for IsPangolinConcrete<T> {
	fn matches_fungible(a: &MultiAsset) -> Option<B> {
		match (&a.id, &a.fun) {
			(Concrete(ref id), Fungible(ref amount))
			if id == &T::get() || id == &SelfLocation::get() => {
				CheckedConversion::checked_from(*amount)
			}
			_ => None,
		}
	}
}
