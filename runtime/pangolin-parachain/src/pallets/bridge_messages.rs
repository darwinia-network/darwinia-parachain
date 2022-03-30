pub use pallet_bridge_messages::Instance1 as WithPangolinMessages;

// --- paritytech ---
use bp_messages::MessageNonce;
use pallet_bridge_messages::Config;
// --- darwinia-netwrok ---
use crate::{
	pangolin_messages::{
		FromPangolinMessageDispatch, FromPangolinMessagePayload, Pangolin,
		PangolinParachainToPangolinParameter, ToPangolinMessagePayload, ToPangolinMessageVerifier,
	},
	*,
};
use bp_pangolin_parachain::{
	AccountIdConverter, MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT,
	MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX, MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX,
};

frame_support::parameter_types! {
	pub RootAccountForPayments: Option<AccountId> = None;
	pub const MaxMessagesToPruneAtOnce: MessageNonce = 8;
	pub const BridgedChainId: bp_runtime::ChainId = bp_runtime::PANGOLIN_CHAIN_ID;
	pub const MaxUnconfirmedMessagesAtInboundLane: MessageNonce = MAX_UNCONFIRMED_MESSAGES_IN_CONFIRMATION_TX;
	pub const MaxUnrewardedRelayerEntriesAtInboundLane: MessageNonce = MAX_UNREWARDED_RELAYERS_IN_CONFIRMATION_TX;
	pub const GetDeliveryConfirmationTransactionFee: Balance = MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT as _;
}

impl Config<WithPangolinMessages> for Runtime {
	type Event = Event;
	type WeightInfo = ();
	type BridgedChainId = BridgedChainId;
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
	type LaneMessageVerifier = ToPangolinMessageVerifier;
	type MessageDeliveryAndDispatchPayment =
		pallet_bridge_messages::instant_payments::InstantCurrencyPayments<
			Runtime,
			WithPangolinMessages,
			Ring,
			GetDeliveryConfirmationTransactionFee,
			RootAccountForPayments,
		>;
	type OnMessageAccepted = ();
	type OnDeliveryConfirmed = ();
	type SourceHeaderChain = Pangolin;
	type MessageDispatch = FromPangolinMessageDispatch;
}
