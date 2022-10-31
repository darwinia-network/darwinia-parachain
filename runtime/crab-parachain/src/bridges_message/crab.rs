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
use dp_common_runtime::FromThisChainMessageVerifier;

/// Message delivery proof for CrabParachain -> Crab messages.
pub type ToCrabMessagesDeliveryProof = FromBridgedChainMessagesDeliveryProof<bp_crab::Hash>;
/// Message proof for Crab -> CrabParachain  messages.
pub type FromCrabMessagesProof = FromBridgedChainMessagesProof<bp_crab::Hash>;

/// Message payload for CrabParachain -> Crab messages.
pub type ToCrabMessagePayload = FromThisChainMessagePayload;
/// Message payload for Crab -> CrabParachain messages.
pub type FromCrabMessagePayload = FromBridgedChainMessagePayload<Call>;

/// Message verifier for CrabParachain -> Crab messages.
pub type ToCrabMessageVerifier<R> =
	FromThisChainMessageVerifier<WithCrabMessageBridge, R, WithCrabFeeMarket>;

/// Call-dispatch based message dispatch for Crab -> CrabParachain messages.
pub type FromCrabMessageDispatch = FromBridgedChainMessageDispatch<
	WithCrabMessageBridge,
	xcm_executor::XcmExecutor<crate::polkadot_xcm::XcmConfig>,
	crate::polkadot_xcm::XcmWeigher,
	// 2 XCM instructions is for simple `Trap(42)` program, coming through bridge
	// (it is prepended with `UniversalOrigin` instruction)
	frame_support::traits::ConstU64<BASE_XCM_WEIGHT_TWICE>,
>;

pub const INITIAL_CRAB_TO_CRAB_PARACHAIN_CONVERSION_RATE: FixedU128 =
	FixedU128::from_inner(FixedU128::DIV);
/// Weight of 2 XCM instructions is for simple `Trap(42)` program, coming through bridge
/// (it is prepended with `UniversalOrigin` instruction). It is used just for simplest manual
/// tests, confirming that we don't break encoding somewhere between.
pub const BASE_XCM_WEIGHT_TWICE: Weight = 2 * crate::xcm_config::BASE_XCM_WEIGHT;

frame_support::parameter_types! {
	/// CrabParachain to Crab conversion rate. Initially we trate both tokens as equal.
	pub storage CrabToCrabParachainConversionRate: FixedU128 = INITIAL_CRAB_TO_CRAB_PARACHAIN_CONVERSION_RATE;
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum CrabParachainToCrabParameter {
	/// The conversion formula we use is: `CrabTokens = CrabParachainTokens *
	/// conversion_rate`.
	CrabToCrabParachainConversionRate(FixedU128),
}
impl Parameter for CrabParachainToCrabParameter {
	fn save(&self) {
		match *self {
			CrabParachainToCrabParameter::CrabToCrabParachainConversionRate(
				ref conversion_rate,
			) => CrabToCrabParachainConversionRate::set(conversion_rate),
		}
	}
}

/// Crab <-> CrabParachain message bridge.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct WithCrabMessageBridge;
impl MessageBridge for WithCrabMessageBridge {
	type BridgedChain = Crab;
	type ThisChain = CrabParachain;

	const BRIDGED_CHAIN_ID: ChainId = CRAB_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bp_crab_parachain::WITH_CRAB_PARACHAIN_MESSAGES_PALLET_NAME;
	const RELAYER_FEE_PERCENT: u32 = 10;
	const THIS_CHAIN_ID: ChainId = CRAB_PARACHAIN_CHAIN_ID;
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct CrabParachain;
impl ChainWithMessages for CrabParachain {
	type AccountId = bp_crab_parachain::AccountId;
	type Balance = bp_crab_parachain::Balance;
	type Hash = bp_crab_parachain::Hash;
	type Signature = bp_crab_parachain::Signature;
	type Signer = bp_crab_parachain::AccountPublic;
	type Weight = Weight;
}
impl ThisChainWithMessages for CrabParachain {
	type Call = Call;
	type Origin = Origin;

	fn is_message_accepted(send_origin: &Self::Origin, lane: &LaneId) -> bool {
		let here_location =
			xcm::v3::MultiLocation::from(crate::xcm_config::UniversalLocation::get());
		match send_origin.caller {
			OriginCaller::PolkadotXcm(pallet_xcm::Origin::Xcm(ref location))
			if *location == here_location =>
				{
					log::trace!(target: "runtime::bridge", "Verifying message sent using XCM pallet");
				},
			_ => {
				// keep in mind that in this case all messages are free (in term of fees)
				// => it's just to keep testing bridge on our test deployments until we'll have a
				// better option
				log::trace!(target: "runtime::bridge", "Verifying message sent using messages pallet");
			},
		}

		*lane == [0, 0, 0, 0] || *lane == [0, 0, 0, 1]
	}

	fn maximal_pending_messages_at_outbound_lane() -> MessageNonce {
		MessageNonce::MAX
	}
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct Crab;
impl ChainWithMessages for Crab {
	type AccountId = bp_crab::AccountId;
	type Balance = bp_crab::Balance;
	type Hash = bp_crab::Hash;
	type Signature = bp_crab::Signature;
	type Signer = bp_crab::AccountPublic;
	type Weight = Weight;
}
impl BridgedChainWithMessages for Crab {
	fn maximal_extrinsic_size() -> u32 {
		bp_crab::Crab::max_extrinsic_size()
	}

	fn verify_dispatch_weight(_message_payload: &[u8]) -> bool {
		true
	}
}
impl TargetHeaderChain<ToCrabMessagePayload, <Self as ChainWithMessages>::AccountId> for Crab {
	type Error = &'static str;
	type MessagesDeliveryProof = ToCrabMessagesDeliveryProof;

	fn verify_message(payload: &ToCrabMessagePayload) -> Result<(), Self::Error> {
		source::verify_chain_message::<WithCrabMessageBridge>(payload)
	}

	fn verify_messages_delivery_proof(
		proof: Self::MessagesDeliveryProof,
	) -> Result<(LaneId, InboundLaneData<bp_crab::AccountId>), Self::Error> {
		source::verify_messages_delivery_proof::<WithCrabMessageBridge, Runtime, WithCrabGrandpa>(
			proof,
		)
	}
}
impl SourceHeaderChain<<Self as ChainWithMessages>::Balance> for Crab {
	type Error = &'static str;
	type MessagesProof = FromCrabMessagesProof;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message<<Self as ChainWithMessages>::Balance>>, Self::Error> {
		target::verify_messages_proof::<WithCrabMessageBridge, Runtime, WithCrabGrandpa>(
			proof,
			messages_count,
		)
	}
}

/// With-Crab bridge
pub struct ToCrabBridge<MB>(PhantomData<MB>);

impl<MB: MessagesBridge<Origin, AccountId, Balance, FromThisChainMessagePayload>> SendXcm
for ToCrabBridge<MB>
{
	type Ticket = (Balance, FromThisChainMessagePayload);

	fn validate(
		dest: &mut Option<MultiLocation>,
		msg: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		let d = dest.take().ok_or(SendError::MissingArgument)?;
		if !matches!(d, MultiLocation { parents: 1, interior: X1(GlobalConsensus(r)) } if r == CrabNetwork::get())
		{
			*dest = Some(d);
			return Err(SendError::NotApplicable)
		};

		let dest: InteriorMultiLocation = CrabNetwork::get().into();
		let here = UniversalLocation::get();
		let route = dest.relative_to(&here);
		let msg = (route, msg.take().unwrap()).encode();

		let fee = estimate_message_dispatch_and_delivery_fee::<WithCrabMessageBridge>(
			&msg,
			WithCrabMessageBridge::RELAYER_FEE_PERCENT,
			None,
		)
			.map_err(SendError::Transport)?;
		let fee_assets = MultiAssets::from((Here, fee));

		Ok(((fee, msg), fee_assets))
	}

	fn deliver(ticket: Self::Ticket) -> Result<XcmHash, SendError> {
		let lane = [0, 0, 0, 0];
		let (fee, msg) = ticket;
		let result = MB::send_message(
			pallet_xcm::Origin::from(MultiLocation::from(UniversalLocation::get())).into(),
			lane,
			msg,
			fee,
		);
		result
			.map(|artifacts| {
				let hash = (lane, artifacts.nonce).using_encoded(sp_io::hashing::blake2_256);
				log::debug!(target: "runtime::bridge", "Sent XCM message {:?}/{} to Crab parachain: {:?}", lane, artifacts.nonce, hash);
				hash
			})
			.map_err(|e| {
				log::debug!(target: "runtime::bridge", "Failed to send XCM message over lane {:?} to Crab Parachain: {:?}", lane, e);
				SendError::Transport("Bridge has rejected the message")
			})
	}
}
