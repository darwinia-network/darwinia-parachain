use bp_pangolin_parachain::EXTRA_STORAGE_PROOF_SIZE;
use pallet_bridge_messages::WeightInfoExt;

impl<T: frame_system::Config> WeightInfoExt
	for crate::weights::pallet_bridge_messages::WeightInfo<T>
{
	fn expected_extra_storage_proof_size() -> u32 {
		EXTRA_STORAGE_PROOF_SIZE
	}
}
