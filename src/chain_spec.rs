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
use crab_redirect_runtime::AuraId;

/// Specialized `ChainSpec` for the `Crab Redirect` parachain runtime.
pub type CrabRedirectChainSpec =
	sc_service::GenericChainSpec<crab_redirect_runtime::GenesisConfig, Extensions>;

type AccountPublic = <Signature as Verify>::Signer;

const CRAB_REDIRECT_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Helper function to generate a crypto pair from seed
fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

fn properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 42.into());
	properties.insert("tokenDecimals".into(), vec![9, 9].into());
	properties.insert("tokenSymbol".into(), vec!["CRING", "CKTON"].into());

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
	let root = AccountId::from(array_bytes::hex2array_unchecked!(
		"0x72819fbc1b93196fa230243947c1726cbea7e33044c7eb6f736ff345561f9e4c",
		32
	));
	let initial_authorities = vec![array_bytes::hex2array_unchecked!(
		"0x72819fbc1b93196fa230243947c1726cbea7e33044c7eb6f736ff345561f9e4c",
		32
	)
	.unchecked_into()];

	crab_redirect_runtime::GenesisConfig {
		frame_system: crab_redirect_runtime::SystemConfig {
			code: crab_redirect_runtime::wasm_binary_unwrap().into(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: Default::default(),
		parachain_info: crab_redirect_runtime::ParachainInfoConfig { parachain_id: id },
		pallet_aura: parachain_runtime::AuraConfig {
			authorities: initial_authorities,
		},
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
			relay_chain: "kusama".into(),
			para_id: id.into(),
		},
	);
}

fn crab_redirect_development_genesis(id: ParaId) -> crab_redirect_runtime::GenesisConfig {
	let root = get_account_id_from_seed::<sr25519::Public>("Alice");
	let initial_authorities = vec![get_from_seed::<AuraId>("Alice")];

	crab_redirect_runtime::GenesisConfig {
		frame_system: crab_redirect_runtime::SystemConfig {
			code: crab_redirect_runtime::wasm_binary_unwrap().into(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: Default::default(),
		parachain_info: crab_redirect_runtime::ParachainInfoConfig { parachain_id: id },
		pallet_aura: parachain_runtime::AuraConfig {
			authorities: initial_authorities,
		},
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
