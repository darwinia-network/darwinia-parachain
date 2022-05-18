// This file is part of Darwinia.
//
// Copyright (C) 2018-2021 Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

//! Autogenerated weights for `pallet_bridge_messages`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2022-04-27, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("pangolin-parachain-dev"), DB CACHE: 1024

// Executed Command:
// ./target/release/darwinia-collator
// benchmark
// --chain
// pangolin-parachain-dev
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// *
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --heap-pages=4096
// --header=./file_header.txt
// --output
// ./runtime/pangolin-parachain/src/weights/

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for `pallet_bridge_messages`.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_bridge_messages::WeightInfo for WeightInfo<T> {
	// Storage: BridgePangolinMessages PalletOperatingMode (r:1 w:0)
	// Storage: BridgePangolinMessages OutboundLanes (r:1 w:1)
	// Storage: FeeMarket AssignedRelayersNumber (r:1 w:0)
	// Storage: FeeMarket AssignedRelayers (r:1 w:0)
	// Storage: System Account (r:2 w:2)
	// Storage: FeeMarket Orders (r:0 w:1)
	// Storage: BridgePangolinMessages OutboundMessages (r:0 w:9)
	fn send_minimal_message_worst_case() -> Weight {
		(89_187_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(13 as Weight))
	}
	// Storage: BridgePangolinMessages PalletOperatingMode (r:1 w:0)
	// Storage: BridgePangolinMessages OutboundLanes (r:1 w:1)
	// Storage: FeeMarket AssignedRelayersNumber (r:1 w:0)
	// Storage: FeeMarket AssignedRelayers (r:1 w:0)
	// Storage: System Account (r:2 w:2)
	// Storage: FeeMarket Orders (r:0 w:1)
	// Storage: BridgePangolinMessages OutboundMessages (r:0 w:9)
	fn send_1_kb_message_worst_case() -> Weight {
		(64_031_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(13 as Weight))
	}
	// Storage: BridgePangolinMessages PalletOperatingMode (r:1 w:0)
	// Storage: BridgePangolinMessages OutboundLanes (r:1 w:1)
	// Storage: FeeMarket AssignedRelayersNumber (r:1 w:0)
	// Storage: FeeMarket AssignedRelayers (r:1 w:0)
	// Storage: System Account (r:2 w:2)
	// Storage: FeeMarket Orders (r:0 w:1)
	// Storage: BridgePangolinMessages OutboundMessages (r:0 w:9)
	fn send_16_kb_message_worst_case() -> Weight {
		(84_128_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(13 as Weight))
	}
	// Storage: BridgePangolinMessages PalletOperatingMode (r:1 w:0)
	// Storage: BridgePangolinMessages OutboundLanes (r:1 w:0)
	// Storage: System Account (r:2 w:2)
	// Storage: BridgePangolinMessages OutboundMessages (r:1 w:1)
	fn maximal_increase_message_fee() -> Weight {
		(5_347_846_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: BridgePangolinMessages PalletOperatingMode (r:1 w:0)
	// Storage: BridgePangolinMessages OutboundLanes (r:1 w:0)
	// Storage: System Account (r:2 w:2)
	// Storage: BridgePangolinMessages OutboundMessages (r:1 w:1)
	fn increase_message_fee(i: u32, ) -> Weight {
		(0 as Weight)
			// Standard Error: 0
			.saturating_add((3_000 as Weight).saturating_mul(i as Weight))
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: BridgePangolinMessages PalletOperatingMode (r:1 w:0)
	// Storage: BridgePangolinGrandpa ImportedHeaders (r:1 w:0)
	// Storage: BridgePangolinMessages InboundLanes (r:1 w:1)
	// Storage: TransactionPayment NextFeeMultiplier (r:1 w:0)
	// Storage: System Account (r:2 w:2)
	fn receive_single_message_proof() -> Weight {
		(80_559_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: BridgePangolinMessages PalletOperatingMode (r:1 w:0)
	// Storage: BridgePangolinGrandpa ImportedHeaders (r:1 w:0)
	// Storage: BridgePangolinMessages InboundLanes (r:1 w:1)
	// Storage: TransactionPayment NextFeeMultiplier (r:1 w:0)
	// Storage: System Account (r:2 w:2)
	fn receive_two_messages_proof() -> Weight {
		(134_576_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: BridgePangolinMessages PalletOperatingMode (r:1 w:0)
	// Storage: BridgePangolinGrandpa ImportedHeaders (r:1 w:0)
	// Storage: BridgePangolinMessages InboundLanes (r:1 w:1)
	// Storage: TransactionPayment NextFeeMultiplier (r:1 w:0)
	// Storage: System Account (r:2 w:2)
	fn receive_single_message_proof_with_outbound_lane_state() -> Weight {
		(83_651_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: BridgePangolinMessages PalletOperatingMode (r:1 w:0)
	// Storage: BridgePangolinGrandpa ImportedHeaders (r:1 w:0)
	// Storage: BridgePangolinMessages InboundLanes (r:1 w:1)
	// Storage: TransactionPayment NextFeeMultiplier (r:1 w:0)
	// Storage: System Account (r:2 w:2)
	fn receive_single_message_proof_1_kb() -> Weight {
		(88_558_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: BridgePangolinMessages PalletOperatingMode (r:1 w:0)
	// Storage: BridgePangolinGrandpa ImportedHeaders (r:1 w:0)
	// Storage: BridgePangolinMessages InboundLanes (r:1 w:1)
	// Storage: TransactionPayment NextFeeMultiplier (r:1 w:0)
	// Storage: System Account (r:2 w:2)
	fn receive_single_message_proof_16_kb() -> Weight {
		(162_448_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(6 as Weight))
			.saturating_add(T::DbWeight::get().writes(3 as Weight))
	}
	// Storage: BridgePangolinMessages PalletOperatingMode (r:1 w:0)
	// Storage: BridgePangolinGrandpa ImportedHeaders (r:1 w:0)
	// Storage: BridgePangolinMessages InboundLanes (r:1 w:1)
	fn receive_single_prepaid_message_proof() -> Weight {
		(56_279_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(3 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: BridgePangolinMessages PalletOperatingMode (r:1 w:0)
	// Storage: BridgePangolinGrandpa ImportedHeaders (r:1 w:0)
	// Storage: BridgePangolinMessages OutboundLanes (r:1 w:1)
	// Storage: FeeMarket Orders (r:1 w:0)
	fn receive_delivery_proof_for_single_message() -> Weight {
		(28_079_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: BridgePangolinMessages PalletOperatingMode (r:1 w:0)
	// Storage: BridgePangolinGrandpa ImportedHeaders (r:1 w:0)
	// Storage: BridgePangolinMessages OutboundLanes (r:1 w:1)
	// Storage: FeeMarket Orders (r:2 w:0)
	fn receive_delivery_proof_for_two_messages_by_single_relayer() -> Weight {
		(32_050_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: BridgePangolinMessages PalletOperatingMode (r:1 w:0)
	// Storage: BridgePangolinGrandpa ImportedHeaders (r:1 w:0)
	// Storage: BridgePangolinMessages OutboundLanes (r:1 w:1)
	// Storage: FeeMarket Orders (r:2 w:0)
	fn receive_delivery_proof_for_two_messages_by_two_relayers() -> Weight {
		(30_665_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(5 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}
