// --- crates.io ---
use codec::{Decode, Encode};
use scale_info::TypeInfo;
// --- paritytech ---
use frame_support::{weights::Weight, RuntimeDebug};
use sp_runtime::{FixedPointNumber, FixedU128};
use sp_std::ops::RangeInclusive;
// --- darwinia-network ---
use crate::*;
use bp_messages::{source_chain::*, target_chain::*, *};
use bp_runtime::*;
use bridge_runtime_common::{
	lanes::*,
	messages::{source::*, target::*, *},
};

/// Message delivery proof for DarwiniaParachain -> Darwinia messages.
pub type ToDarwiniaMessagesDeliveryProof = FromBridgedChainMessagesDeliveryProof<bp_darwinia::Hash>;
/// Message proof for Darwinia -> DarwiniaParachain  messages.
pub type FromDarwiniaMessagesProof = FromBridgedChainMessagesProof<bp_darwinia::Hash>;

/// Message payload for DarwiniaParachain -> Darwinia messages.
pub type ToDarwiniaMessagePayload = FromThisChainMessagePayload<WithDarwiniaMessageBridge>;
/// Message payload for Darwinia -> DarwiniaParachain messages.
pub type FromDarwiniaMessagePayload = FromBridgedChainMessagePayload<WithDarwiniaMessageBridge>;

/// Message verifier for DarwiniaParachain -> Darwinia messages.
pub type ToDarwiniaMessageVerifier<R> =
	FromThisChainMessageVerifier<WithDarwiniaMessageBridge, R, WithDarwiniaFeeMarket>;

/// Encoded Darwinia Call as it comes from Darwinia.
pub type FromDarwiniaEncodedCall = FromBridgedChainEncodedMessageCall<Call>;

/// Call-dispatch based message dispatch for Darwinia -> DarwiniaParachain messages.
pub type FromDarwiniaMessageDispatch =
	FromBridgedChainMessageDispatch<WithDarwiniaMessageBridge, Runtime, Ring, WithDarwiniaDispatch>;

pub const INITIAL_DARWINIA_TO_DARWINIA_PARACHAIN_CONVERSION_RATE: FixedU128 =
	FixedU128::from_inner(FixedU128::DIV);

frame_support::parameter_types! {
	/// DarwiniaParachain to Darwinia conversion rate. Initially we trate both tokens as equal.
	pub storage DarwiniaToDarwiniaParachainConversionRate: FixedU128 = INITIAL_DARWINIA_TO_DARWINIA_PARACHAIN_CONVERSION_RATE;
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum DarwiniaParachainToDarwiniaParameter {
	/// The conversion formula we use is: `DarwiniaTokens = DarwiniaParachainTokens *
	/// conversion_rate`.
	DarwiniaToDarwiniaParachainConversionRate(FixedU128),
}
impl Parameter for DarwiniaParachainToDarwiniaParameter {
	fn save(&self) {
		match *self {
			DarwiniaParachainToDarwiniaParameter::DarwiniaToDarwiniaParachainConversionRate(
				ref conversion_rate,
			) => DarwiniaToDarwiniaParachainConversionRate::set(conversion_rate),
		}
	}
}

/// Darwinia <-> DarwiniaParachain message bridge.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct WithDarwiniaMessageBridge;
impl MessageBridge for WithDarwiniaMessageBridge {
	type BridgedChain = Darwinia;
	type ThisChain = DarwiniaParachain;

	const BRIDGED_CHAIN_ID: ChainId = DARWINIA_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bp_darwinia_parachain::WITH_DARWINIA_PARACHAIN_MESSAGES_PALLET_NAME;
	const RELAYER_FEE_PERCENT: u32 = 10;
	const THIS_CHAIN_ID: ChainId = DARWINIA_PARACHAIN_CHAIN_ID;
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct DarwiniaParachain;
impl ChainWithMessages for DarwiniaParachain {
	type AccountId = bp_darwinia_parachain::AccountId;
	type Balance = bp_darwinia_parachain::Balance;
	type Hash = bp_darwinia_parachain::Hash;
	type Signature = bp_darwinia_parachain::Signature;
	type Signer = bp_darwinia_parachain::AccountPublic;
	type Weight = Weight;
}
impl ThisChainWithMessages for DarwiniaParachain {
	type Call = Call;
	type Origin = Origin;

	fn is_message_accepted(_send_origin: &Self::Origin, lane: &LaneId) -> bool {
		*lane == DARWINIA_DARWINIA_PARACHAIN_LANE
	}

	fn maximal_pending_messages_at_outbound_lane() -> MessageNonce {
		MessageNonce::MAX
	}
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct Darwinia;
impl ChainWithMessages for Darwinia {
	type AccountId = bp_darwinia::AccountId;
	type Balance = bp_darwinia::Balance;
	type Hash = bp_darwinia::Hash;
	type Signature = bp_darwinia::Signature;
	type Signer = bp_darwinia::AccountPublic;
	type Weight = Weight;
}
impl BridgedChainWithMessages for Darwinia {
	fn maximal_extrinsic_size() -> u32 {
		bp_darwinia::Darwinia::max_extrinsic_size()
	}

	fn message_weight_limits(_message_payload: &[u8]) -> RangeInclusive<Self::Weight> {
		let upper_limit = target::maximal_incoming_message_dispatch_weight(
			bp_darwinia::Darwinia::max_extrinsic_weight(),
		);
		0..=upper_limit
	}
}
impl TargetHeaderChain<ToDarwiniaMessagePayload, <Self as ChainWithMessages>::AccountId>
	for Darwinia
{
	type Error = &'static str;
	type MessagesDeliveryProof = ToDarwiniaMessagesDeliveryProof;

	fn verify_message(payload: &ToDarwiniaMessagePayload) -> Result<(), Self::Error> {
		source::verify_chain_message::<WithDarwiniaMessageBridge>(payload)
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<(LaneId, InboundLaneData<bp_darwinia::AccountId>), Self::Error> {
		source::verify_messages_delivery_proof::<
			WithDarwiniaMessageBridge,
			Runtime,
			WithDarwiniaGrandpa,
		>(proof)
	}
}
impl SourceHeaderChain<<Self as ChainWithMessages>::Balance> for Darwinia {
	type Error = &'static str;
	type MessagesProof = FromDarwiniaMessagesProof;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message<<Self as ChainWithMessages>::Balance>>, Self::Error> {
		target::verify_messages_proof::<WithDarwiniaMessageBridge, Runtime, WithDarwiniaGrandpa>(
			proof,
			messages_count,
		)
	}
}
