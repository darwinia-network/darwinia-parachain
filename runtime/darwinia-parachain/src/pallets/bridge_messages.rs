pub use pallet_bridge_messages::Instance1 as WithDarwiniaMessages;

// --- darwinia-network ---
use crate::*;
use bp_messages::{source_chain::SenderOrigin, MessageNonce};
use bp_runtime::{ChainId, DARWINIA_CHAIN_ID};
use pallet_bridge_messages::Config;
use pallet_fee_market::s2s::{
	FeeMarketMessageAcceptedHandler, FeeMarketMessageConfirmedHandler, FeeMarketPayment,
};

impl SenderOrigin<AccountId> for Origin {
	fn linked_account(&self) -> Option<AccountId> {
		match self.caller {
			OriginCaller::system(frame_system::RawOrigin::Signed(ref submitter)) =>
				Some(submitter.clone()),
			_ => None,
		}
	}
}

frame_support::parameter_types! {
	pub const MaxMessagesToPruneAtOnce: MessageNonce = 8;
	pub const BridgedChainId: ChainId = DARWINIA_CHAIN_ID;
	pub const MaxUnconfirmedMessagesAtInboundLane: MessageNonce =
		bp_darwinia::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: MessageNonce =
		bp_darwinia::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const GetDeliveryConfirmationTransactionFee: Balance =
		bp_darwinia::MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT as _;
	pub RootAccountForPayments: Option<AccountId> = None;
}

impl Config<WithDarwiniaMessages> for Runtime {
	type AccountIdConverter = bp_darwinia_parachain::AccountIdConverter;
	type BridgedChainId = BridgedChainId;
	type Event = Event;
	type InboundMessageFee = bp_darwinia::Balance;
	type InboundPayload = bm_darwinia::FromDarwiniaMessagePayload;
	type InboundRelayer = bp_darwinia::AccountId;
	type LaneMessageVerifier = bm_darwinia::ToDarwiniaMessageVerifier<Self>;
	type MaxMessagesToPruneAtOnce = MaxMessagesToPruneAtOnce;
	type MaxUnconfirmedMessagesAtInboundLane = MaxUnconfirmedMessagesAtInboundLane;
	type MaxUnrewardedRelayerEntriesAtInboundLane = MaxUnrewardedRelayerEntriesAtInboundLane;
	type MessageDeliveryAndDispatchPayment = FeeMarketPayment<Self, WithDarwiniaFeeMarket, Ring>;
	type MessageDispatch = bm_darwinia::FromDarwiniaMessageDispatch;
	type OnDeliveryConfirmed =
		(FromDarwiniaIssuing, FeeMarketMessageConfirmedHandler<Self, WithDarwiniaFeeMarket>);
	type OnMessageAccepted = FeeMarketMessageAcceptedHandler<Self, WithDarwiniaFeeMarket>;
	type OutboundMessageFee = bp_darwinia_parachain::Balance;
	type OutboundPayload = bm_darwinia::ToDarwiniaMessagePayload;
	type Parameter = bm_darwinia::DarwiniaParachainToDarwiniaParameter;
	type SourceHeaderChain = bm_darwinia::Darwinia;
	type TargetHeaderChain = bm_darwinia::Darwinia;
	type WeightInfo = ();
}
