pub use pallet_bridge_messages::Instance1 as WithCrabMessages;

// --- darwinia-network ---
use crate::*;
use bp_messages::{source_chain::SenderOrigin, MessageNonce};
use bp_runtime::{ChainId, CRAB_CHAIN_ID};
use pallet_bridge_messages::Config;
use pallet_fee_market::s2s::{
	FeeMarketMessageAcceptedHandler, FeeMarketMessageConfirmedHandler, FeeMarketPayment,
};

impl SenderOrigin<AccountId> for RuntimeOrigin {
	fn linked_account(&self) -> Option<AccountId> {
		// XCM deals wit fees in our deployments
		None
	}
}

frame_support::parameter_types! {
	pub const MaxMessagesToPruneAtOnce: MessageNonce = 8;
	pub const BridgedChainId: ChainId = CRAB_CHAIN_ID;
	pub const MaxUnconfirmedMessagesAtInboundLane: MessageNonce =
		bp_crab::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: MessageNonce =
		bp_crab::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const GetDeliveryConfirmationTransactionFee: Balance =
		bp_crab::MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT.ref_time() as _;
	pub RootAccountForPayments: Option<AccountId> = None;
}

impl Config<WithCrabMessages> for Runtime {
	type AccountIdConverter = bp_crab_parachain::AccountIdConverter;
	type BridgedChainId = BridgedChainId;
	type InboundMessageFee = bp_crab::Balance;
	type InboundPayload = bm_crab::FromCrabMessagePayload;
	type InboundRelayer = bp_crab::AccountId;
	type LaneMessageVerifier = bm_crab::ToCrabMessageVerifier<Self>;
	type MaxMessagesToPruneAtOnce = MaxMessagesToPruneAtOnce;
	type MaxUnconfirmedMessagesAtInboundLane = MaxUnconfirmedMessagesAtInboundLane;
	type MaxUnrewardedRelayerEntriesAtInboundLane = MaxUnrewardedRelayerEntriesAtInboundLane;
	type MessageDeliveryAndDispatchPayment = FeeMarketPayment<Self, WithCrabFeeMarket, Ring>;
	type MessageDispatch = bm_crab::FromCrabMessageDispatch;
	type OnDeliveryConfirmed =
		(FromCrabIssuing, FeeMarketMessageConfirmedHandler<Self, WithCrabFeeMarket>);
	type OnMessageAccepted = FeeMarketMessageAcceptedHandler<Self, WithCrabFeeMarket>;
	type OutboundMessageFee = bp_crab_parachain::Balance;
	type OutboundPayload = bm_crab::ToCrabMessagePayload;
	type Parameter = bm_crab::CrabParachainToCrabParameter;
	type RuntimeEvent = RuntimeEvent;
	type SourceHeaderChain = bm_crab::Crab;
	type TargetHeaderChain = bm_crab::Crab;
	type WeightInfo = ();
}
