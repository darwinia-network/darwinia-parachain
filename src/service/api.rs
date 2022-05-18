// --- darwinia-network ---
use super::*;

/// A set of APIs that darwinia-like runtimes must implement.
pub trait RuntimeApiCollection:
	cumulus_primitives_core::CollectCollationInfo<Block>
	+ sp_api::ApiExt<Block, StateBackend = StateBackend>
	+ sp_api::Metadata<Block>
	+ sp_block_builder::BlockBuilder<Block>
	+ sp_consensus_aura::AuraApi<Block, AuraId>
	+ sp_offchain::OffchainWorkerApi<Block>
	+ sp_session::SessionKeys<Block>
	+ sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
	+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
	+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
{
}
impl<Api> RuntimeApiCollection for Api where
	Api: cumulus_primitives_core::CollectCollationInfo<Block>
		+ sp_api::ApiExt<Block, StateBackend = StateBackend>
		+ sp_api::Metadata<Block>
		+ sp_block_builder::BlockBuilder<Block>
		+ sp_consensus_aura::AuraApi<Block, AuraId>
		+ sp_offchain::OffchainWorkerApi<Block>
		+ sp_session::SessionKeys<Block>
		+ sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
		+ substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>
		+ pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>
{
}
