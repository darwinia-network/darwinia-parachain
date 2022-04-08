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

//! Parachain-specific RPCs implementation.

pub use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};

// --- std ---
use std::sync::Arc;
// --- crates.io ---
use jsonrpc_core::IoHandler;
// --- paritytech ---
use sc_rpc::Metadata;
use sp_blockchain::Error as BlockChainError;
// --- darwinia-network ---
use dc_primitives::{AccountId, Balance, Nonce, OpaqueBlock as Block};

/// A type representing all RPC extensions.
pub type RpcExtension = IoHandler<Metadata>;

/// Full client dependencies
pub struct FullDeps<C, P> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all RPC extensions.
pub fn create_full<C, P>(deps: FullDeps<C, P>) -> RpcExtension
where
	C: 'static
		+ Send
		+ Sync
		+ sp_api::ProvideRuntimeApi<Block>
		+ sp_blockchain::HeaderBackend<Block>
		+ sc_client_api::AuxStore
		+ sp_blockchain::HeaderMetadata<Block, Error = BlockChainError>,
	C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
		+ sp_block_builder::BlockBuilder<Block>
		+ pallet_fee_market_rpc::FeeMarketRuntimeApi<Block, Balance>,

	P: 'static + Send + Sync + sc_transaction_pool_api::TransactionPool,
{
	// --- paritytech ---
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};
	use substrate_frame_rpc_system::{FullSystem, SystemApi};
	// --- darwinia-network ---
	use pallet_fee_market_rpc::*;

	let mut io = IoHandler::default();
	let FullDeps {
		client,
		pool,
		deny_unsafe,
	} = deps;

	io.extend_with(SystemApi::to_delegate(FullSystem::new(
		client.clone(),
		pool,
		deny_unsafe,
	)));
	io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
		client.clone(),
	)));
	io.extend_with(FeeMarketApi::to_delegate(FeeMarket::new(client.clone())));

	io
}
