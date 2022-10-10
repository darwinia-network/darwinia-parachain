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

// --- crates.io ---
use ethereum::TransactionV2 as Transaction;
use ethereum_types::{H160, U256};
use ethabi::{
	param_type::ParamType, token::Token, Bytes, Function, Param, Result as AbiResult,
	StateMutability,
};
// --- paritytech ---
use sp_runtime::AccountId32;

/// The address prefix for dvm address
const ADDR_PREFIX: &[u8] = b"dvm:";

fn checksum_of(account_id: &[u8; 32]) -> u8 {
	account_id[1..31].iter().fold(account_id[0], |sum, &byte| sum ^ byte)
}

fn derive_substrate_address(address: &H160) -> AccountId32 {
    let mut raw_account = [0u8; 32];

    raw_account[0..4].copy_from_slice(ADDR_PREFIX);
    raw_account[11..31].copy_from_slice(&address[..]);
    raw_account[31] = checksum_of(&raw_account);

    raw_account.into()
}

pub fn new_ethereum_transaction(
    chain_id: u32,
    contractAddress: H160,
    gasLimit: U256,
    input: Vec<u8>,
    ) -> Result<TransactionV2, DispatchError> {
    let sig = TransactionSignature::new(
        chain_id * 2 + 36,
        H256::from_slice(&sig[0..32]),
        H256::from_slice(&sig[32..64]),
        )?;

    Ok(TransactionV2::Legacy(LegacyTransaction {
        nonce: 0,
        gas_price: 0,
        gas_limit: gasLimit,
        action: TransactionAction::Call(contractAddress),
        value: 0,
        input: input,
        signature: sig,
    }))
}

pub struct ToParachainBacking;
impl ToParachainBacking {
    pub fn encode_unlock_from_remote(
        recipient: H160,
        amount: U256,
    ) -> AbiResult<Bytes> {
        let inputs = vec![
			Param {
				name: "recipient".into(),
				kind: ParamType::Address,
				internal_type: Some("address".into()),
			},
			Param {
				name: "amount".into(),
				kind: ParamType::Uint(256),
				internal_type: Some("uint256".into()),
			},
		];
        
        #[allow(deprecated)]
		Function {
			name: "unlockFromRemote".into(),
			inputs,
			outputs: vec![],
			constant: false,
			state_mutability: StateMutability::NonPayable,
		}
		.encode_input(
			vec![
				Token::Address(recipient),
				Token::Uint(amount),
			]
			.as_slice(),
		)
    }

    pub fn encode_handle_unlock_failure_from_remote(
        nonce: u64,
    ) -> AbiResult<Bytes> {
        let inputs = vec![
			Param {
				name: "nonce".into(),
				kind: ParamType::Uint(64),
				internal_type: Some("uint64".into()),
			},
		];
        
        #[allow(deprecated)]
		Function {
			name: "handleUnlockFailureFromRemote".into(),
			inputs,
			outputs: vec![],
			constant: false,
			state_mutability: StateMutability::NonPayable,
		}
		.encode_input(
			vec![
				Token::Uint(nonce.into()),
			]
			.as_slice(),
		)
    }
}

