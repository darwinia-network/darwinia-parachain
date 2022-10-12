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

use sp_std::marker::PhantomData;
use xcm::{latest::MultiLocation, prelude::*};
use xcm_executor::traits::Convert;

/// The address prefix for dvm address
const ADDR_PREFIX: &[u8] = b"dvm:";

pub struct AccountKey20Derive<AccountId>(PhantomData<AccountId>);
impl<AccountId: From<[u8; 32]> + Into<[u8; 32]> + Clone> Convert<MultiLocation, AccountId>
	for AccountKey20Derive<AccountId>
{
	fn convert(location: MultiLocation) -> Result<AccountId, MultiLocation> {
		let key = match location {
			MultiLocation { parents: 0, interior: X1(AccountKey20 { key, network: _ }) } => key,
			MultiLocation {
				parents: 1,
				interior: X2(Parachain(_), AccountKey20 { key, network: _ }),
			} => key,
			_ => return Err(location),
		};
		let mut raw_account = [0u8; 32];

		raw_account[0..4].copy_from_slice(ADDR_PREFIX);
		raw_account[11..31].copy_from_slice(&key[..]);
		raw_account[31] = checksum_of(&raw_account);
		Ok(raw_account.into())
	}

	fn reverse(who: AccountId) -> Result<MultiLocation, AccountId> {
		Ok(AccountId32 { id: who.into(), network: Any }.into())
	}
}

fn checksum_of(account_id: &[u8; 32]) -> u8 {
	account_id[1..31].iter().fold(account_id[0], |sum, &byte| sum ^ byte)
}
