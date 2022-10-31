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
	lanes::PANGOLIN_PANGOLIN_PARACHAIN_LANE,
	messages::{source::*, target::*, *},
};
use dp_common_runtime::FromThisChainMessageVerifier;

/// Message delivery proof for PangolinParachain -> Pangolin messages.
pub type ToPangolinMessagesDeliveryProof = FromBridgedChainMessagesDeliveryProof<bp_pangolin::Hash>;
/// Message proof for Pangolin -> PangolinParachain  messages.
pub type FromPangolinMessagesProof = FromBridgedChainMessagesProof<bp_pangolin::Hash>;

/// Message payload for PangolinParachain -> Pangolin messages.
pub type ToPangolinMessagePayload = FromThisChainMessagePayload;
/// Message payload for Pangolin -> PangolinParachain messages.
pub type FromPangolinMessagePayload = FromBridgedChainMessagePayload<Call>;

/// Message verifier for PangolinParachain -> Pangolin messages.
pub type ToPangolinMessageVerifier<R> =
	FromThisChainMessageVerifier<WithPangolinMessageBridge, R, WithPangolinFeeMarket>;

/// Call-dispatch based message dispatch for Pangolin -> PangolinParachain messages.
pub type FromPangolinMessageDispatch = FromBridgedChainMessageDispatch<
	WithPangolinMessageBridge,
	xcm_executor::XcmExecutor<crate::polkadot_xcm::XcmConfig>,
	crate::polkadot_xcm::XcmWeigher,
	// 2 XCM instructions is for simple `Trap(42)` program, coming through bridge
	// (it is prepended with `UniversalOrigin` instruction)
	frame_support::traits::ConstU64<BASE_XCM_WEIGHT_TWICE>,
>;

pub const INITIAL_PANGOLIN_TO_PANGOLIN_PARACHAIN_CONVERSION_RATE: FixedU128 =
	FixedU128::from_inner(FixedU128::DIV);
/// Weight of 2 XCM instructions is for simple `Trap(42)` program, coming through bridge
/// (it is prepended with `UniversalOrigin` instruction). It is used just for simplest manual
/// tests, confirming that we don't break encoding somewhere between.
pub const BASE_XCM_WEIGHT_TWICE: Weight = 2 * crate::xcm_config::BASE_XCM_WEIGHT;

frame_support::parameter_types! {
	/// PangolinParachain to Pangolin conversion rate. Initially we trate both tokens as equal.
	pub storage PangolinToPangolinParachainConversionRate: FixedU128 = INITIAL_PANGOLIN_TO_PANGOLIN_PARACHAIN_CONVERSION_RATE;
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum PangolinParachainToPangolinParameter {
	/// The conversion formula we use is: `PangolinTokens = PangolinParachainTokens *
	/// conversion_rate`.
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

/// Pangolin <-> PangolinParachain message bridge.
#[derive(Clone, Copy, RuntimeDebug)]
pub struct WithPangolinMessageBridge;
impl MessageBridge for WithPangolinMessageBridge {
	type BridgedChain = Pangolin;
	type ThisChain = PangolinParachain;

	const BRIDGED_CHAIN_ID: ChainId = PANGOLIN_CHAIN_ID;
	const BRIDGED_MESSAGES_PALLET_NAME: &'static str =
		bp_pangolin_parachain::WITH_PANGOLIN_PARACHAIN_MESSAGES_PALLET_NAME;
	const RELAYER_FEE_PERCENT: u32 = 10;
	const THIS_CHAIN_ID: ChainId = PANGOLIN_PARACHAIN_CHAIN_ID;
}

#[derive(Clone, Copy, RuntimeDebug)]
pub struct PangolinParachain;
impl ChainWithMessages for PangolinParachain {
	type AccountId = bp_pangolin_parachain::AccountId;
	type Balance = bp_pangolin_parachain::Balance;
	type Hash = bp_pangolin_parachain::Hash;
	type Signature = bp_pangolin_parachain::Signature;
	type Signer = bp_pangolin_parachain::AccountPublic;
	type Weight = Weight;
}
impl ThisChainWithMessages for PangolinParachain {
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
pub struct Pangolin;
impl ChainWithMessages for Pangolin {
	type AccountId = bp_pangolin::AccountId;
	type Balance = bp_pangolin::Balance;
	type Hash = bp_pangolin::Hash;
	type Signature = bp_pangolin::Signature;
	type Signer = bp_pangolin::AccountPublic;
	type Weight = Weight;
}
impl BridgedChainWithMessages for Pangolin {
	fn maximal_extrinsic_size() -> u32 {
		bp_pangolin::Pangolin::max_extrinsic_size()
	}

	fn verify_dispatch_weight(_message_payload: &[u8]) -> bool {
		true
	}
}
impl TargetHeaderChain<ToPangolinMessagePayload, <Self as ChainWithMessages>::AccountId>
	for Pangolin
{
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
			WithPangolinGrandpa,
		>(proof)
	}
}
impl SourceHeaderChain<<Self as ChainWithMessages>::Balance> for Pangolin {
	type Error = &'static str;
	type MessagesProof = FromPangolinMessagesProof;

	fn verify_messages_proof(
		proof: Self::MessagesProof,
		messages_count: u32,
	) -> Result<ProvedMessages<Message<<Self as ChainWithMessages>::Balance>>, Self::Error> {
		target::verify_messages_proof::<WithPangolinMessageBridge, Runtime, WithPangolinGrandpa>(
			proof,
			messages_count,
		)
	}
}

/// The s2s backing pallet index in the pangoro chain runtime.
pub const PANGOLIN_S2S_BACKING_PALLET_INDEX: u8 = 65;

/// With-pangolin bridge
pub struct ToPangolinBridge<MB>(PhantomData<MB>);

impl<MB: MessagesBridge<Origin, AccountId, Balance, FromThisChainMessagePayload>> SendXcm
	for ToPangolinBridge<MB>
{
	type Ticket = (Balance, FromThisChainMessagePayload);

	fn validate(
		dest: &mut Option<MultiLocation>,
		msg: &mut Option<Xcm<()>>,
	) -> SendResult<Self::Ticket> {
		let d = dest.take().ok_or(SendError::MissingArgument)?;
		if !matches!(d, MultiLocation { parents: 1, interior: X1(GlobalConsensus(r)) } if r == PangolinNetwork::get())
		{
			*dest = Some(d);
			return Err(SendError::NotApplicable);
		};

		let dest: InteriorMultiLocation = PangolinNetwork::get().into();
		let here = UniversalLocation::get();
		let route = dest.relative_to(&here);
		let msg = (route, msg.take().unwrap()).encode();

		let fee = estimate_message_dispatch_and_delivery_fee::<WithPangolinMessageBridge>(
			&msg,
			WithPangolinMessageBridge::RELAYER_FEE_PERCENT,
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
				log::debug!(target: "runtime::bridge", "Sent XCM message {:?}/{} to Pangolin parachain: {:?}", lane, artifacts.nonce, hash);
				hash
			})
			.map_err(|e| {
				log::debug!(target: "runtime::bridge", "Failed to send XCM message over lane {:?} to Pangolin Parachain: {:?}", lane, e);
				SendError::Transport("Bridge has rejected the message")
			})
	}
}
