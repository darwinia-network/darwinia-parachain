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
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

//! Primitives used by the Parachain

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs)]

// --- paritytech ---
use sp_core::H256;
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	MultiAddress, MultiSignature, OpaqueExtrinsic,
};

/// An index to a block.
/// 32-bits will allow for 136 years of blocks assuming 1 block per second.
pub type BlockNumber = u32;

/// Hashing algorithm used by the chain.
pub type Hashing = BlakeTwo256;

/// A hash of some data used by the relay chain.
pub type Hash = H256;

/// An instant or duration in time.
pub type Moment = u64;

/// Alias to type for a signature for a transaction on the relay chain. This allows one of several
/// kinds of underlying crypto to be used, so isn't a fixed size when encoded.
pub type Signature = MultiSignature;

/// Alias to the public key used for this chain, actually a `MultiSigner`. Like the signature, this
/// also isn't a fixed size when encoded, as different cryptos have different size public keys.
pub type AccountPublic = <Signature as Verify>::Signer;

/// Alias to the opaque account ID type for this chain, actually a `AccountId32`. This is always
/// 32 bytes.
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them.
pub type AccountIndex = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Index of a transaction in the relay chain. 32-bit should be plenty.
pub type Nonce = u32;

/// The balance of an account.
/// 128-bits (or 38 significant decimal figures) will allow for 10m currency (10^7) at a resolution
/// to all for one second's worth of an annualised 50% reward be paid to a unit holder (10^11 unit
/// denomination), or 10^18 total atomic units, to grow at 50%/year for 51 years (10^9 multiplier)
/// for an eventual total of 10^27 units (27 significant decimal figures).
/// We round denomination to 10^12 (12 sdf), and leave the other redundancy at the upper end so
/// that 32 bits may be multiplied with a balance in 128 bits without worrying about overflow.
pub type Balance = u128;

/// Header type.
pub type Header = generic::Header<BlockNumber, Hashing>;

/// Block type.
pub type OpaqueBlock = generic::Block<Header, OpaqueExtrinsic>;

/// `1` in `u128`.
pub const WEI: Balance = 1;
/// `1_000` in `u128`.
pub const K_WEI: Balance = 1_000 * WEI;
/// `1_000_000` in `u128`.
pub const M_WEI: Balance = 1_000 * K_WEI;
/// `1_000_000_000` in `u128`.
pub const G_WEI: Balance = 1_000 * M_WEI;
/// `1_000_000_000_000` in `u128`.
pub const MICRO_COIN: Balance = 1_000 * G_WEI;
/// `1_000_000_000_000_000` in `u128`.
pub const MILLI_COIN: Balance = 1_000 * MICRO_COIN;
/// `1_000_000_000_000_000_000` in `u128`.
pub const COIN: Balance = 1_000 * MILLI_COIN;

/// Balance required for the xcm unit weight
pub const WEIGHT_FEE: Balance = G_WEI;

/// Block time of Darwinia Parachain.
pub const MILLISECS_PER_BLOCK: Moment = 12000;

/// Minute in Darwinia Parachain.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);

/// Hour in Darwinia Parachain.
pub const HOURS: BlockNumber = 60 * MINUTES;
/// Day in Darwinia Parachain.
pub const DAYS: BlockNumber = 24 * HOURS;

/// Slot duration in Darwinia Parachain.
pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

/// Deposit calculator for Darwinia Parachain.
pub const fn darwinia_deposit(items: u32, bytes: u32) -> Balance {
	(items as Balance) * 20 * COIN + (bytes as Balance) * 100 * MICRO_COIN
}

/// Deposit calculator for Crab Parachain.
pub const fn crab_deposit(items: u32, bytes: u32) -> Balance {
	(items as Balance) * 20 * MILLI_COIN + (bytes as Balance) * 100 * G_WEI
}

/// Deposit calculator for Pangolin Parachain.
pub const fn pangolin_deposit(items: u32, bytes: u32) -> Balance {
	(items as Balance) * 20 * MILLI_COIN + (bytes as Balance) * 100 * G_WEI
}
