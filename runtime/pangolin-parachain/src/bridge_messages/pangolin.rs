// --- core ---
use core::marker::PhantomData;
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
use bp_messages::{source_chain::TargetHeaderChain, target_chain::*, *};
use bp_runtime::{Chain, ChainId, PANGOLIN_CHAIN_ID, PANGOLIN_PARACHAIN_CHAIN_ID};
use bridge_runtime_common::messages::{self, source::*, target::*, *};
use pallet_bridge_messages::EXPECTED_DEFAULT_MESSAGE_LENGTH;

/// Message delivery proof for PangolinParachain -> Pangolin messages.
type ToPangolinMessagesDeliveryProof = FromBridgedChainMessagesDeliveryProof<bp_pangolin::Hash>;
/// Message proof for Pangolin -> PangolinParachain  messages.
type FromPangolinMessagesProof = FromBridgedChainMessagesProof<bp_pangolin::Hash>;

/// Message payload for PangolinParachain -> Pangolin messages.
pub type ToPangolinMessagePayload = FromThisChainMessagePayload<WithPangolinMessageBridge>;
/// Message payload for Pangolin -> PangolinParachain messages.
pub type FromPangolinMessagePayload = FromBridgedChainMessagePayload<WithPangolinMessageBridge>;

/// Message verifier for PangolinParachain -> Pangolin messages.
pub type ToPangolinMessageVerifier<R> = FromThisChainMessageVerifier<WithPangolinMessageBridge, R>;

/// Encoded Pangolin Call as it comes from Pangolin.
pub type FromPangolinEncodedCall = FromBridgedChainEncodedMessageCall<Call>;

/// Call-dispatch based message dispatch for Pangolin -> PangolinParachain messages.
pub type FromPangolinMessageDispatch =
	FromBridgedChainMessageDispatch<WithPangolinMessageBridge, Runtime, Ring, WithPangolinDispatch>;

pub const INITIAL_PANGOLIN_TO_PANGOLIN_PARACHAIN_CONVERSION_RATE: FixedU128 =
	FixedU128::from_inner(FixedU128::DIV);

/// Identifier of bridge between pangolin and pangolin parachain.
pub const PANGOLIN_PANGOLIN_PARACHAIN_LANE: [u8; 4] = *b"pali";

frame_support::parameter_types! {
	/// PangolinParachain to Pangolin conversion rate. Initially we trate both tokens as equal.
	pub storage PangolinToPangolinParachainConversionRate: FixedU128 = INITIAL_PANGOLIN_TO_PANGOLIN_PARACHAIN_CONVERSION_RATE;
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum PangolinParachainToPangolinParameter {
	/// The conversion formula we use is: `PangolinTokens = PangolinParachainTokens * conversion_rate`.
	PangolinToPangolinParachainConversionRate(FixedU128),
}
impl Parameter for PangolinParachainToPangolinParameter {
	fn save(&self) {
		match *self {
			PangolinParachainToPangolinParameter::PangolinToPangolinParachainConversionRate(
				ref conversion_rate,
			) => PangolinToPangolinParachainConversionRate::set(conversion_rate),
		}
	}
}

use bp_messages::source_chain::{LaneMessageVerifier, Sender};
/// Message verifier that is doing all basic checks.
///
/// This verifier assumes following:
///
/// - all message lanes are equivalent, so all checks are the same;
/// - messages are being dispatched using `pallet-bridge-dispatch` pallet on the target chain.
///
/// Following checks are made:
///
/// - message is rejected if its lane is currently blocked;
/// - message is rejected if there are too many pending (undelivered) messages at the outbound
///   lane;
/// - check that the sender has rights to dispatch the call on target chain using provided
///   dispatch origin;
/// - check that the sender has paid enough funds for both message delivery and dispatch.
#[derive(RuntimeDebug)]
pub struct FromThisChainMessageVerifier<B, R>(PhantomData<(B, R)>);
impl<B, R>
	LaneMessageVerifier<
		AccountIdOf<ThisChain<B>>,
		FromThisChainMessagePayload<B>,
		BalanceOf<ThisChain<B>>,
	> for FromThisChainMessageVerifier<B, R>
where
	B: MessageBridge,
	R: pallet_fee_market::Config,
	AccountIdOf<ThisChain<B>>: Clone + PartialEq,
	pallet_fee_market::RingBalance<R>: From<BalanceOf<ThisChain<B>>>,
{
	type Error = &'static str;

	fn verify_message(
		submitter: &Sender<AccountIdOf<ThisChain<B>>>,
		delivery_and_dispatch_fee: &BalanceOf<ThisChain<B>>,
		lane: &LaneId,
		lane_outbound_data: &OutboundLaneData,
		payload: &FromThisChainMessagePayload<B>,
	) -> Result<(), Self::Error> {
		// reject message if lane is blocked
		if !ThisChain::<B>::is_outbound_lane_enabled(lane) {
			return Err(OUTBOUND_LANE_DISABLED);
		}

		// reject message if there are too many pending messages at this lane
		let max_pending_messages = ThisChain::<B>::maximal_pending_messages_at_outbound_lane();
		let pending_messages = lane_outbound_data
			.latest_generated_nonce
			.saturating_sub(lane_outbound_data.latest_received_nonce);
		if pending_messages > max_pending_messages {
			return Err(TOO_MANY_PENDING_MESSAGES);
		}

		// Do the dispatch-specific check. We assume that the target chain uses
		// `Dispatch`, so we verify the message accordingly.
		pallet_bridge_dispatch::verify_message_origin(submitter, payload)
			.map_err(|_| BAD_ORIGIN)?;

		// Do the delivery_and_dispatch_fee. We assume that the delivery and dispatch fee always
		// greater than the fee market provided fee.
		if let Some(market_fee) = pallet_fee_market::Pallet::<R>::market_fee() {
			let message_fee: pallet_fee_market::RingBalance<R> =
				(*delivery_and_dispatch_fee).into();

			// compare with actual fee paid
			if message_fee < market_fee {
				return Err(TOO_LOW_FEE);
			}
		} else {
			const NO_MARKET_FEE: &str = "The fee market are not ready for accepting messages.";

			return Err(NO_MARKET_FEE);
		}

		Ok(())
	}
}

/// Pangolin <-> PangolinParachain message bridge.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct WithPangolinMessageBridge;
impl MessageBridge for WithPangolinMessageBridge {
	type ThisChain = PangolinParachain;
	type BridgedChain = Pangolin;

	const RELAYER_FEE_PERCENT: u32 = 10;
	const THIS_CHAIN_ID: ChainId = PANGOLIN_PARACHAIN_CHAIN_ID;
	const BRIDGED_CHAIN_ID: ChainId = PANGOLIN_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bp_pangolin_parachain::WITH_PANGOLIN_PARACHAIN_MESSAGES_PALLET_NAME;

	fn bridged_balance_to_this_balance(bridged_balance: BalanceOf<BridgedChain<Self>>) -> Balance {
		Balance::try_from(
			PangolinToPangolinParachainConversionRate::get().saturating_mul_int(bridged_balance),
		)
		.unwrap_or(Balance::MAX)
	}
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct PangolinParachain;
impl ChainWithMessages for PangolinParachain {
	type Hash = Hash;
	type AccountId = AccountId;
	type Signer = AccountPublic;
	type Signature = Signature;
	type Weight = Weight;
	type Balance = Balance;
}
impl ThisChainWithMessages for PangolinParachain {
	type Call = Call;

	fn is_outbound_lane_enabled(lane: &LaneId) -> bool {
		*lane == [0, 0, 0, 0] || *lane == [0, 0, 0, 1] || *lane == PANGOLIN_PANGOLIN_PARACHAIN_LANE
	}

	fn maximal_pending_messages_at_outbound_lane() -> MessageNonce {
		MessageNonce::MAX
	}

	fn estimate_delivery_confirmation_transaction() -> MessageTransaction<Weight> {
		let inbound_data_size = InboundLaneData::<AccountId>::encoded_size_hint(
			bp_pangolin_parachain::MAXIMAL_ENCODED_ACCOUNT_ID_SIZE,
			1,
			1,
		)
		.unwrap_or(u32::MAX);

		MessageTransaction {
			dispatch_weight:
				bp_pangolin_parachain::MAX_SINGLE_MESSAGE_DELIVERY_CONFIRMATION_TX_WEIGHT,
			size: inbound_data_size
				.saturating_add(bp_pangolin_parachain::EXTRA_STORAGE_PROOF_SIZE)
				.saturating_add(bp_pangolin_parachain::TX_EXTRA_BYTES),
		}
	}

	fn transaction_payment(transaction: MessageTransaction<Weight>) -> Balance {
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
impl ChainWithMessages for Pangolin {
	type Hash = bp_pangolin::Hash;
	type AccountId = bp_pangolin::AccountId;
	type Signer = bp_pangolin::AccountPublic;
	type Signature = bp_pangolin::Signature;
	type Weight = Weight;
	type Balance = bp_pangolin::Balance;
}
impl BridgedChainWithMessages for Pangolin {
	fn maximal_extrinsic_size() -> u32 {
		bp_pangolin::Pangolin::max_extrinsic_size()
	}

	fn message_weight_limits(_message_payload: &[u8]) -> RangeInclusive<Self::Weight> {
		let upper_limit = target::maximal_incoming_message_dispatch_weight(
			bp_pangolin::Pangolin::max_extrinsic_weight(),
		);
		0..=upper_limit
	}

	fn estimate_delivery_transaction(
		message_payload: &[u8],
		include_pay_dispatch_fee_cost: bool,
		message_dispatch_weight: Weight,
	) -> MessageTransaction<Weight> {
		let message_payload_len = u32::try_from(message_payload.len()).unwrap_or(u32::MAX);
		let extra_bytes_in_payload = Weight::from(message_payload_len)
			.saturating_sub(EXPECTED_DEFAULT_MESSAGE_LENGTH.into());

		MessageTransaction {
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

	fn transaction_payment(transaction: MessageTransaction<Weight>) -> bp_pangolin::Balance {
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
	) -> Result<(LaneId, InboundLaneData<bp_pangolin::AccountId>), Self::Error> {
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
	) -> Result<ProvedMessages<Message<bp_pangolin::Balance>>, Self::Error> {
		target::verify_messages_proof::<
			WithPangolinMessageBridge,
			Runtime,
			crate::WithPangolinGrandpa,
		>(proof, messages_count)
	}
}
