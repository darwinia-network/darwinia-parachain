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
use ethabi::{
	param_type::ParamType, token::Token, Bytes, Function, Param, Result as AbiResult,
	StateMutability,
};
use ethereum::{
	LegacyTransaction, TransactionAction, TransactionSignature, TransactionV2 as Transaction,
};
use ethereum_types::{H160, H256, U256};
// --- paritytech ---
use frame_support::pallet_prelude::PhantomData;
use sp_runtime::DispatchError;
use sp_std::{boxed::Box, vec, vec::Vec};

/// A trait for converting from Substrate account_id to Ethereum address.
pub trait DeriveEthereumAddress<AccountId> {
	fn derive_ethereum_address(account: AccountId) -> H160;
}

pub struct ConcatConverter<AccountId>(PhantomData<AccountId>);

impl<AccountId> DeriveEthereumAddress<AccountId> for ConcatConverter<AccountId>
where
	AccountId: Into<[u8; 32]>,
{
	fn derive_ethereum_address(account: AccountId) -> H160 {
		let bytes: [u8; 32] = account.into();
		H160::from_slice(&bytes[11..31])
	}
}

pub fn new_ethereum_transaction(
	chain_id: u64,
	contract_address: H160,
	gas_limit: U256,
	input: Vec<u8>,
) -> Result<Transaction, DispatchError> {
	let sig = TransactionSignature::new(
		chain_id * 2 + 36,
		H256::from_slice(&[55u8; 32]),
		H256::from_slice(&[55u8; 32]),
	)
	.unwrap();

	Ok(Transaction::Legacy(LegacyTransaction {
		nonce: U256::zero(),
		gas_price: U256::zero(),
		gas_limit,
		action: TransactionAction::Call(contract_address),
		value: U256::zero(),
		input,
		signature: sig,
	}))
}

pub struct ToParachainBacking;
impl ToParachainBacking {
	pub fn encode_unlock_from_remote(
		recipient: H160,
		amount: U256,
		prun_nonces: Vec<u64>,
		min_reserved_burn_nonce: u64,
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
			Param {
				name: "prunNonces".into(),
				kind: ParamType::Array(Box::new(ParamType::Uint(64))),
				internal_type: Some("uint64[]".into()),
			},
			Param {
				name: "minReservedBurnNonce".into(),
				kind: ParamType::Uint(64),
				internal_type: Some("uint64".into()),
			},
		];

		#[allow(deprecated)]
		Function {
			name: "unlockFromRemote".into(),
			inputs,
			outputs: vec![],
			constant: Some(false),
			state_mutability: StateMutability::NonPayable,
		}
		.encode_input(
			vec![
				Token::Address(recipient),
				Token::Uint(amount),
				Token::Array(prun_nonces.iter().map(|n| Token::Uint((*n).into())).collect()),
				Token::Uint(min_reserved_burn_nonce.into()),
			]
			.as_slice(),
		)
	}

	pub fn encode_handle_unlock_failure_from_remote(
		nonce: u64,
		prun_nonces: Vec<u64>,
		min_reserved_burn_nonce: u64,
	) -> AbiResult<Bytes> {
		let inputs = vec![
			Param {
				name: "nonce".into(),
				kind: ParamType::Uint(64),
				internal_type: Some("uint64".into()),
			},
			Param {
				name: "prunNonces".into(),
				kind: ParamType::Array(Box::new(ParamType::Uint(64))),
				internal_type: Some("uint64[]".into()),
			},
			Param {
				name: "minReservedBurnNonce".into(),
				kind: ParamType::Uint(64),
				internal_type: Some("uint64".into()),
			},
		];

		#[allow(deprecated)]
		Function {
			name: "handleUnlockFailureFromRemote".into(),
			inputs,
			outputs: vec![],
			constant: Some(false),
			state_mutability: StateMutability::NonPayable,
		}
		.encode_input(
			vec![
				Token::Uint(nonce.into()),
				Token::Array(prun_nonces.iter().map(|n| Token::Uint((*n).into())).collect()),
				Token::Uint(min_reserved_burn_nonce.into()),
			]
			.as_slice(),
		)
	}
}
