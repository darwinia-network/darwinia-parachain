pub use pallet_bridge_messages::Instance1 as WithPangolinMessages;

// --- darwinia-network ---
use crate::{bridges_message::bm_pangolin, *};
use bp_messages::MessageNonce;
use bp_runtime::{ChainId, PANGOLIN_CHAIN_ID};
use pallet_bridge_messages::Config;
use pallet_fee_market::s2s::{
	FeeMarketMessageAcceptedHandler, FeeMarketMessageConfirmedHandler, FeeMarketPayment,
};

frame_support::parameter_types! {
	pub const MaxMessagesToPruneAtOnce: MessageNonce = 8;
	pub const BridgedChainId: ChainId = PANGOLIN_CHAIN_ID;
	pub const MaxUnconfirmedMessagesAtInboundLane: MessageNonce =
		bp_pangolin::MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: MessageNonce =
		bp_pangolin::MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const GetDeliveryConfirmationTransactionFee: Balance =
		bp_pangolin::MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT as _;
	pub RootAccountForPayments: Option<AccountId> = None;
}

impl Config<WithPangolinMessages> for Runtime {
	type Event = Event;
	type WeightInfo = weights::pallet_bridge_messages::WeightInfo<Runtime>;
	type Parameter = bm_pangolin::PangolinParachainToPangolinParameter;
	type MaxMessagesToPruneAtOnce = MaxMessagesToPruneAtOnce;
	type MaxUnrewardedRelayerEntriesAtInboundLane = MaxUnrewardedRelayerEntriesAtInboundLane;
	type MaxUnconfirmedMessagesAtInboundLane = MaxUnconfirmedMessagesAtInboundLane;

	type OutboundPayload = bm_pangolin::ToPangolinMessagePayload;
	type OutboundMessageFee = Balance;

	type InboundPayload = bm_pangolin::FromPangolinMessagePayload;
	type InboundMessageFee = bp_pangolin::Balance;
	type InboundRelayer = bp_pangolin::AccountId;

	type AccountIdConverter = bp_pangolin_parachain::AccountIdConverter;

	type TargetHeaderChain = bm_pangolin::Pangolin;
	type LaneMessageVerifier = bm_pangolin::ToPangolinMessageVerifier<Self>;
	type MessageDeliveryAndDispatchPayment = FeeMarketPayment<
		Runtime,
		WithPangolinMessages,
		Ring,
		GetDeliveryConfirmationTransactionFee,
		RootAccountForPayments,
	>;

	type OnMessageAccepted = FeeMarketMessageAcceptedHandler<Self>;
	type OnDeliveryConfirmed = (FeeMarketMessageConfirmedHandler<Self>,);

	type SourceHeaderChain = bm_pangolin::Pangolin;
	type MessageDispatch = bm_pangolin::FromPangolinMessageDispatch;
	type BridgedChainId = BridgedChainId;
}
