// --- crates.io ---
use codec::{Decode, Encode};
use scale_info::TypeInfo;
// --- paritytech ---
use frame_support::{
	weights::{DispatchClass, Weight},
	RuntimeDebug,
};
use sp_runtime::{traits::Zero, FixedPointNumber, FixedU128};
use sp_std::ops::RangeInclusive;
// --- darwinia-network ---
use crate::*;
use bp_messages::{
	source_chain::TargetHeaderChain, target_chain::SourceHeaderChain,
	Parameter as MessagesParameter,
};
use bp_runtime::{Chain, PANGOLIN_CHAIN_ID, PANGOLIN_PARACHAIN_CHAIN_ID};
use bridge_runtime_common::messages::{
	self,
	source::{
		self, FromBridgedChainMessagesDeliveryProof, FromThisChainMessagePayload,
		FromThisChainMessageVerifier,
	},
	target::{
		self, FromBridgedChainEncodedMessageCall, FromBridgedChainMessageDispatch,
		FromBridgedChainMessagePayload, FromBridgedChainMessagesProof,
	},
	MessageBridge,
};
use pallet_bridge_messages::EXPECTED_DEFAULT_MESSAGE_LENGTH;

/// Identifier of PangolinParachain in the relay chain.
pub const PANGOLIN_PARACHAIN_ID: u32 = 2071;

/// Identifier of bridge between pangolin and pangolin parachain.
pub const PANGOLIN_PANGOLIN_PARACHAIN_LANE: [u8; 4] = *b"pali";

/// Message verifier for PangolinParachain -> Pangolin messages.
pub type ToPangolinMessageVerifier = FromThisChainMessageVerifier<WithPangolinMessageBridge>;
/// Message payload for PangolinParachain -> Pangolin messages.
pub type ToPangolinMessagePayload = FromThisChainMessagePayload<WithPangolinMessageBridge>;

/// Message payload for Pangolin -> PangolinParachain messages.
pub type FromPangolinMessagePayload = FromBridgedChainMessagePayload<WithPangolinMessageBridge>;
/// Call-dispatch based message dispatch for PangolinParachain -> Pangolin messages.
pub type FromPangolinMessageDispatch =
	FromBridgedChainMessageDispatch<WithPangolinMessageBridge, Runtime, Ring, WithPangolinDispatch>;

/// Message proof for Pangolin -> PangolinParachain  messages.
type FromPangolinMessagesProof = FromBridgedChainMessagesProof<bp_pangolin::Hash>;
/// Message delivery proof for PangolinParachain -> Pangolin messages.
type ToPangolinMessagesDeliveryProof = FromBridgedChainMessagesDeliveryProof<bp_pangolin::Hash>;

/// Encoded Pangolin Call as it comes from Pangolin.
pub type FromPangolinEncodedCall = FromBridgedChainEncodedMessageCall<crate::Call>;

pub const INITIAL_PANGOLIN_TO_PANGOLIN_PARACHAIN_CONVERSION_RATE: FixedU128 =
	FixedU128::from_inner(FixedU128::DIV);

frame_support::parameter_types! {
	/// PangolinParachain to Pangolin conversion rate. Initially we trate both tokens as equal.
	pub storage PangolinToPangolinParachainConversionRate: FixedU128 = INITIAL_PANGOLIN_TO_PANGOLIN_PARACHAIN_CONVERSION_RATE;
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum PangolinParachainToPangolinParameter {
	/// The conversion formula we use is: `PangolinTokens = PangolinParachainTokens * conversion_rate`.
	PangolinToPangolinParachainConversionRate(FixedU128),
}

impl MessagesParameter for PangolinParachainToPangolinParameter {
	fn save(&self) {
		match *self {
			PangolinParachainToPangolinParameter::PangolinToPangolinParachainConversionRate(
				ref conversion_rate,
			) => PangolinToPangolinParachainConversionRate::set(conversion_rate),
		}
	}
}

/// Pangolin <-> PangolinParachain message bridge.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct WithPangolinMessageBridge;
impl MessageBridge for WithPangolinMessageBridge {
	const RELAYER_FEE_PERCENT: u32 = 10;
	const THIS_CHAIN_ID: bp_runtime::ChainId = PANGOLIN_PARACHAIN_CHAIN_ID;
	const BRIDGED_CHAIN_ID: bp_runtime::ChainId = PANGOLIN_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bp_pangolin_parachain::WITH_PANGOLIN_PARACHAIN_MESSAGES_PALLET_NAME;

	type ThisChain = PangolinParachain;

	type BridgedChain = Pangolin;

	fn bridged_balance_to_this_balance(
		bridged_balance: messages::BalanceOf<messages::BridgedChain<Self>>,
	) -> Balance {
		Balance::try_from(
			PangolinToPangolinParachainConversionRate::get().saturating_mul_int(bridged_balance),
		)
		.unwrap_or(Balance::MAX)
	}
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct PangolinParachain;
impl messages::ChainWithMessages for PangolinParachain {
	type Hash = Hash;
	type AccountId = AccountId;
	type Signer = AccountPublic;
	type Signature = Signature;
	type Weight = Weight;
	type Balance = Balance;
}
impl messages::ThisChainWithMessages for PangolinParachain {
	type Call = Call;

	fn is_outbound_lane_enabled(lane: &bp_messages::LaneId) -> bool {
		*lane == [0, 0, 0, 0] || *lane == [0, 0, 0, 1] || *lane == PANGOLIN_PANGOLIN_PARACHAIN_LANE
	}

	fn maximal_pending_messages_at_outbound_lane() -> bp_messages::MessageNonce {
		bp_messages::MessageNonce::MAX
	}

	fn estimate_delivery_confirmation_transaction() -> messages::MessageTransaction<Weight> {
		let inbound_data_size = bp_messages::InboundLaneData::<AccountId>::encoded_size_hint(
			bp_pangolin_parachain::MAXIMAL_ENCODED_ACCOUNT_ID_SIZE,
			1,
			1,
		)
		.unwrap_or(u32::MAX);

		messages::MessageTransaction {
			dispatch_weight:
				bp_pangolin_parachain::MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT,
			size: inbound_data_size
				.saturating_add(bp_pangolin_parachain::EXTRA_STORAGE_PROOF_SIZE)
				.saturating_add(bp_pangolin_parachain::TX_EXTRA_BYTES),
		}
	}

	fn transaction_payment(transaction: messages::MessageTransaction<Weight>) -> Balance {
		// in our testnets, both per-byte fee and weight-to-fee are 1:1
		messages::transaction_payment(
			RuntimeBlockWeights::get()
				.get(DispatchClass::Normal)
				.base_extrinsic,
			1,
			FixedU128::zero(),
			|weight| weight as _,
			transaction,
		)
	}
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct Pangolin;
impl messages::ChainWithMessages for Pangolin {
	type Hash = bp_pangolin::Hash;
	type AccountId = bp_pangolin::AccountId;
	type Signer = bp_pangolin::AccountPublic;
	type Signature = bp_pangolin::Signature;
	type Weight = Weight;
	type Balance = bp_pangolin::Balance;
}
impl messages::BridgedChainWithMessages for Pangolin {
	fn maximal_extrinsic_size() -> u32 {
		bp_pangolin::Pangolin::max_extrinsic_size()
	}

	fn message_weight_limits(_message_payload: &[u8]) -> RangeInclusive<Self::Weight> {
		let upper_limit = messages::target::maximal_incoming_message_dispatch_weight(
			bp_pangolin::Pangolin::max_extrinsic_weight(),
		);
		0..=upper_limit
	}

	fn estimate_delivery_transaction(
		message_payload: &[u8],
		include_pay_dispatch_fee_cost: bool,
		message_dispatch_weight: Weight,
	) -> messages::MessageTransaction<Weight> {
		let message_payload_len = u32::try_from(message_payload.len()).unwrap_or(u32::MAX);
		let extra_bytes_in_payload = Weight::from(message_payload_len)
			.saturating_sub(EXPECTED_DEFAULT_MESSAGE_LENGTH.into());

		messages::MessageTransaction {
			dispatch_weight: extra_bytes_in_payload
				.saturating_mul(bp_pangolin::ADDITIONAL_MESSAGE_BYTE_DELIVERY_WEIGHT)
				.saturating_add(bp_pangolin::DEFAULT_MESSAGE_DELIVERY_TX_WEIGHT)
				.saturating_add(message_dispatch_weight)
				.saturating_sub(if include_pay_dispatch_fee_cost {
					0
				} else {
					bp_pangolin::PAY_INBOUND_DISPATCH_FEE_WEIGHT
				}),
			size: message_payload_len
				.saturating_add(bp_pangolin::EXTRA_STORAGE_PROOF_SIZE)
				.saturating_add(bp_pangolin::TX_EXTRA_BYTES),
		}
	}

	fn transaction_payment(
		transaction: messages::MessageTransaction<Weight>,
	) -> bp_pangolin::Balance {
		// in our testnets, both per-byte fee and weight-to-fee are 1:1
		messages::transaction_payment(
			bp_pangolin::RuntimeBlockWeights::get()
				.get(DispatchClass::Normal)
				.base_extrinsic,
			1,
			FixedU128::zero(),
			|weight| weight as _,
			transaction,
		)
	}
}
impl TargetHeaderChain<ToPangolinMessagePayload, bp_pangolin::AccountId> for Pangolin {
	type Error = &'static str;

	type MessagesDeliveryProof = ToPangolinMessagesDeliveryProof;

	fn verify_message(payload: &ToPangolinMessagePayload) -> Result<(), Self::Error> {
		source::verify_chain_message::<WithPangolinMessageBridge>(payload)
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<
		(
			bp_messages::LaneId,
			bp_messages::InboundLaneData<bp_pangolin::AccountId>,
		),
		Self::Error,
	> {
		source::verify_messages_delivery_proof::<
			WithPangolinMessageBridge,
			Runtime,
			crate::WithPangolinGrandpa,
		>(proof)
	}
}
impl SourceHeaderChain<bp_pangolin::Balance> for Pangolin {
	type Error = &'static str;

	type MessagesProof = FromPangolinMessagesProof;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<
		bp_messages::target_chain::ProvedMessages<bp_messages::Message<bp_pangolin::Balance>>,
		Self::Error,
	> {
		target::verify_messages_proof::<
			WithPangolinMessageBridge,
			Runtime,
			crate::WithPangolinGrandpa,
		>(proof, messages_count)
	}
}
