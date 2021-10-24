#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_utility.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_utility::WeightInfo for WeightInfo<T> {
	fn batch(c: u32) -> Weight {
		(16_177_000 as Weight)
			// Standard Error: 0
			.saturating_add((4_582_000 as Weight).saturating_mul(c as Weight))
	}
	fn as_derivative() -> Weight {
		(7_848_000 as Weight)
	}
	fn batch_all(c: u32) -> Weight {
		(17_745_000 as Weight)
			// Standard Error: 0
			.saturating_add((4_578_000 as Weight).saturating_mul(c as Weight))
	}
}
