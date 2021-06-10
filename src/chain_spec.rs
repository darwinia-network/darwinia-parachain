// This file is part of Darwinia.
//
// Copyright (C) 2018-2021 Darwinia Network
// SPDX-License-Identifier: GPL-3.0
//
// Darwinia is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Darwinia is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Darwinia. If not, see <https://www.gnu.org/licenses/>.

// --- crates ---
use serde::{Deserialize, Serialize};
// --- parity ---
use cumulus_primitives_core::ParaId;
// --- parity ---
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use sp_core::{crypto::UncheckedInto, sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
// --- darwinia ---
use crab_redirect_primitives::{AccountId, Signature};
use crab_redirect_runtime::{AuraId, SessionKeys};

/// Specialized `ChainSpec` for the `Crab Redirect` parachain runtime.
pub type CrabRedirectChainSpec =
	sc_service::GenericChainSpec<crab_redirect_runtime::GenesisConfig, Extensions>;

type AccountPublic = <Signature as Verify>::Signer;

const CRAB_REDIRECT_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Helper function to generate a crypto pair from seed
fn get_pair_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_pair_from_seed::<TPublic>(seed)).into_account()
}

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_pair_from_seed::<AuraId>(seed)
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
fn crab_redirect_session_keys(keys: AuraId) -> SessionKeys {
	SessionKeys { aura: keys }
}

fn properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 42.into());
	properties.insert("tokenDecimals".into(), 9.into());
	properties.insert("tokenSymbol".into(), "CRING".into());

	properties
}

pub fn crab_redirect_build_spec_config_of(id: ParaId) -> CrabRedirectChainSpec {
	return CrabRedirectChainSpec::from_genesis(
		"Crab Redirect",
		"Crab Redirect",
		ChainType::Live,
		move || crab_redirect_build_spec_genesis(id),
		vec![],
		Some(
			TelemetryEndpoints::new(vec![(CRAB_REDIRECT_TELEMETRY_URL.to_string(), 0)])
				.expect("Crab Redirect telemetry url is valid; qed"),
		),
		None,
		Some(properties()),
		Extensions {
			relay_chain: "kusama".into(),
			para_id: id.into(),
		},
	);
}

fn crab_redirect_build_spec_genesis(id: ParaId) -> crab_redirect_runtime::GenesisConfig {
	let root = array_bytes::hex_into_unchecked(
		"0x72819fbc1b93196fa230243947c1726cbea7e33044c7eb6f736ff345561f9e4c",
	);
	let invulnerables = [
		// Denny
		"0x7e8672b2c2ad0904ba6137de480eaa3b9476042f3f2ae08da033c4ccf2272d5a",
		"0xbe7e6c55feca7ffbfd961c93acdf1bc68bea91d758fb8da92f65c66bbf12ea74",
		// Xavier
		"0xb4f7f03bebc56ebe96bc52ea5ed3159d45a0ce3a8d7f082983c33ef133274747",
		// Way
		"0xea0f4185dd32c1278d7bbd3cdd2fbaec3ca29921a88c04c175401a0668d88e66",
	]
	.iter()
	.map(|hex| {
		(
			array_bytes::hex_into_unchecked(hex),
			array_bytes::hex2array_unchecked(hex).unchecked_into(),
		)
	})
	.collect::<Vec<_>>();

	crab_redirect_runtime::GenesisConfig {
		frame_system: crab_redirect_runtime::SystemConfig {
			code: crab_redirect_runtime::wasm_binary_unwrap().into(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: Default::default(),
		parachain_info: crab_redirect_runtime::ParachainInfoConfig { parachain_id: id },
		pallet_collator_selection: crab_redirect_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: 0,
			..Default::default()
		},
		pallet_session: crab_redirect_runtime::SessionConfig {
			keys: invulnerables
				.iter()
				.cloned()
				.map(|(acc, aura)| {
					(
						acc.clone(),                      // account id
						acc.clone(),                      // validator id
						crab_redirect_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		pallet_aura: Default::default(),
		cumulus_pallet_aura_ext: Default::default(),
		pallet_sudo: crab_redirect_runtime::SudoConfig { key: root },
		cumulus_pallet_parachain_system: Default::default(),
	}
}

pub fn crab_redirect_development_config_of(id: ParaId) -> CrabRedirectChainSpec {
	return CrabRedirectChainSpec::from_genesis(
		"Crab Redirect",
		"Crab Redirect",
		ChainType::Development,
		move || crab_redirect_development_genesis(id),
		vec![],
		None,
		None,
		Some(properties()),
		Extensions {
			relay_chain: "kusama-dev".into(),
			para_id: id.into(),
		},
	);
}

fn crab_redirect_development_genesis(id: ParaId) -> crab_redirect_runtime::GenesisConfig {
	let root = get_account_id_from_seed::<sr25519::Public>("Alice");
	let invulnerables = vec![(
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		get_collator_keys_from_seed("Alice"),
	)];

	crab_redirect_runtime::GenesisConfig {
		frame_system: crab_redirect_runtime::SystemConfig {
			code: crab_redirect_runtime::wasm_binary_unwrap().into(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: Default::default(),
		parachain_info: crab_redirect_runtime::ParachainInfoConfig { parachain_id: id },
		pallet_collator_selection: crab_redirect_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: 0,
			..Default::default()
		},
		pallet_session: crab_redirect_runtime::SessionConfig {
			keys: invulnerables
				.iter()
				.cloned()
				.map(|(acc, aura)| {
					(
						acc.clone(),                      // account id
						acc.clone(),                      // validator id
						crab_redirect_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		pallet_aura: Default::default(),
		cumulus_pallet_aura_ext: Default::default(),
		pallet_sudo: crab_redirect_runtime::SudoConfig { key: root },
		cumulus_pallet_parachain_system: Default::default(),
	}
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}
impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}
