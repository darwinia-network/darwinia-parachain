pub use pallet_bridge_dispatch::Instance1 as WithCrabDispatch;

// --- paritytech ---
use sp_runtime::transaction_validity::TransactionValidityError;
// --- darwinia-network ---
use crate::*;
use bp_message_dispatch::{CallValidate, IntoDispatchOrigin as IntoDispatchOriginT};
use bp_messages::{LaneId, MessageNonce};
use pallet_bridge_dispatch::Config;

pub struct CallValidator;
impl CallValidate<bp_crab_parachain::AccountId, Origin, Call> for CallValidator {
	fn check_receiving_before_dispatch(
		_relayer_account: &bp_crab_parachain::AccountId,
		_call: &Call,
	) -> Result<(), &'static str> {
		Ok(())
	}

	fn call_validate(
		_relayer_account: &bp_crab_parachain::AccountId,
		_origin: &Origin,
		_call: &Call,
	) -> Result<(), TransactionValidityError> {
		Ok(())
	}
}

pub struct IntoDispatchOrigin;
impl IntoDispatchOriginT<bp_crab_parachain::AccountId, Call, Origin> for IntoDispatchOrigin {
	fn into_dispatch_origin(id: &bp_crab_parachain::AccountId, _: &Call) -> Origin {
		frame_system::RawOrigin::Signed(id.clone()).into()
	}
}

impl Config<WithCrabDispatch> for Runtime {
	type AccountIdConverter = bp_crab_parachain::AccountIdConverter;
	type BridgeMessageId = (LaneId, MessageNonce);
	type Call = Call;
	type CallValidator = CallValidator;
	type EncodedCall = bm_crab::FromCrabEncodedCall;
	type Event = Event;
	type IntoDispatchOrigin = IntoDispatchOrigin;
	type SourceChainAccountId = bp_crab::AccountId;
	type TargetChainAccountPublic = bp_crab_parachain::AccountPublic;
	type TargetChainSignature = bp_crab_parachain::Signature;
}
