pub use pallet_bridge_messages::Instance1 as WithPangolinMessages;

// --- darwinia-network ---
use crate::{bridge_messages::pangolin::*, *};
use bp_messages::MessageNonce;
use bp_pangolin_parachain::{
	AccountIdConverter, MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT,
	MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX, MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX,
};
use bp_runtime::{ChainId, PANGOLIN_CHAIN_ID};
use pallet_bridge_messages::Config;
use pallet_fee_market::s2s::{
	FeeMarketMessageAcceptedHandler, FeeMarketMessageConfirmedHandler, FeeMarketPayment,
};

frame_support::parameter_types! {
	pub RootAccountForPayments: Option<AccountId> = None;
	pub const MaxMessagesToPruneAtOnce: MessageNonce = 8;
	pub const BridgedChainId: ChainId = PANGOLIN_CHAIN_ID;
	pub const MaxUnconfirmedMessagesAtInboundLane: MessageNonce = MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: MessageNonce = MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const GetDeliveryConfirmationTransactionFee: Balance = MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT as _;
}

impl Config<WithPangolinMessages> for Runtime {
	type Event = Event;
	type WeightInfo = ();
	type Parameter = PangolinParachainToPangolinParameter;
	type MaxMessagesToPruneAtOnce = MaxMessagesToPruneAtOnce;
	type MaxUnrewardedRelayerEntriesAtInboundLane = MaxUnrewardedRelayerEntriesAtInboundLane;
	type MaxUnconfirmedMessagesAtInboundLane = MaxUnconfirmedMessagesAtInboundLane;

	type OutboundPayload = ToPangolinMessagePayload;
	type OutboundMessageFee = Balance;

	type InboundPayload = FromPangolinMessagePayload;
	type InboundMessageFee = bp_pangolin_parachain::Balance;
	type InboundRelayer = bp_pangolin_parachain::AccountId;

	type AccountIdConverter = AccountIdConverter;

	type TargetHeaderChain = Pangolin;
	type LaneMessageVerifier = ToPangolinMessageVerifier<Self>;
	type MessageDeliveryAndDispatchPayment = FeeMarketPayment<
		Runtime,
		WithPangolinMessages,
		Ring,
		GetDeliveryConfirmationTransactionFee,
		RootAccountForPayments,
	>;

	type OnMessageAccepted = FeeMarketMessageAcceptedHandler<Self>;
	type OnDeliveryConfirmed = (FeeMarketMessageConfirmedHandler<Self>,);

	type SourceHeaderChain = Pangolin;
	type MessageDispatch = FromPangolinMessageDispatch;
	type BridgedChainId = BridgedChainId;
}
