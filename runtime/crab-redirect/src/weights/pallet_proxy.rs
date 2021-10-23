#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_proxy.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_proxy::WeightInfo for WeightInfo<T> {
	fn proxy(p: u32) -> Weight {
		(27_318_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((208_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
	}
	fn proxy_announced(a: u32, p: u32) -> Weight {
		(60_665_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((677_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 2_000
			.saturating_add((197_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn remove_announcement(a: u32, p: u32) -> Weight {
		(39_455_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((687_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 2_000
			.saturating_add((3_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn reject_announcement(a: u32, p: u32) -> Weight {
		(39_411_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((686_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 2_000
			.saturating_add((3_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn announce(a: u32, p: u32) -> Weight {
		(54_386_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((677_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 2_000
			.saturating_add((194_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	fn add_proxy(p: u32) -> Weight {
		(37_411_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((298_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn remove_proxy(p: u32) -> Weight {
		(36_658_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((332_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn remove_proxies(p: u32) -> Weight {
		(34_893_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((209_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn anonymous(p: u32) -> Weight {
		(51_243_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((44_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	fn kill_anonymous(p: u32) -> Weight {
		(37_188_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((208_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
