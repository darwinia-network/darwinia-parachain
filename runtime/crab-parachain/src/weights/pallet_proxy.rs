// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

//! Autogenerated weights for `pallet_proxy`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-05-13, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("crab-parachain-dev"), DB CACHE: 1024

// Executed Command:
// target/release/darwinia-collator
// benchmark
// --header
// .maintain/lincense-header
// --execution
// wasm
// --heap-pages
// 4096
// --steps
// 50
// --repeat
// 20
// --chain
// crab-parachain-dev
// --output
// runtime/crab-parachain/src/weights
// --extrinsic
// *
// --pallet
// *

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_proxy`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_proxy::WeightInfo for WeightInfo<T> {
	// Storage: Proxy Proxies (r:1 w:0)
	fn proxy(p: u32, ) -> Weight {
		(12_837_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((119_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
	}
	// Storage: Proxy Proxies (r:1 w:0)
	// Storage: Proxy Announcements (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn proxy_announced(a: u32, p: u32, ) -> Weight {
		(29_770_000 as Weight)
			// Standard Error: 9_000
			.saturating_add((305_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 9_000
			.saturating_add((115_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Proxy Announcements (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn remove_announcement(a: u32, p: u32, ) -> Weight {
		(19_521_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((332_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 1_000
			.saturating_add((5_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Proxy Announcements (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn reject_announcement(a: u32, p: u32, ) -> Weight {
		(19_484_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((333_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 1_000
			.saturating_add((6_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Proxy Proxies (r:1 w:0)
	// Storage: Proxy Announcements (r:1 w:1)
	// Storage: System Account (r:1 w:1)
	fn announce(a: u32, p: u32, ) -> Weight {
		(27_603_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((318_000 as Weight).saturating_mul(a as Weight))
			// Standard Error: 2_000
			.saturating_add((111_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: Proxy Proxies (r:1 w:1)
	fn add_proxy(p: u32, ) -> Weight {
		(22_641_000 as Weight)
			// Standard Error: 2_000
			.saturating_add((160_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Proxy Proxies (r:1 w:1)
	fn remove_proxy(p: u32, ) -> Weight {
		(18_360_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((166_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Proxy Proxies (r:1 w:1)
	fn remove_proxies(p: u32, ) -> Weight {
		(20_491_000 as Weight)
			// Standard Error: 15_000
			.saturating_add((19_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: unknown [0x3a65787472696e7369635f696e646578] (r:1 w:0)
	// Storage: Proxy Proxies (r:1 w:1)
	fn anonymous(p: u32, ) -> Weight {
		(26_368_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((13_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: Proxy Proxies (r:1 w:1)
	fn kill_anonymous(p: u32, ) -> Weight {
		(19_932_000 as Weight)
			// Standard Error: 1_000
			.saturating_add((118_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
