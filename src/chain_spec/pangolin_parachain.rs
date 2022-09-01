// This file is part of Darwinia.
//
// Copyright (C) 2018-2022 Darwinia Network
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

#[cfg(not(feature = "alpha"))]
mod network {
	pub const NAME: &str = "Pangolin Parachain";
	pub const ID: &str = "pangolin_parachain";

	pub const BOOT_NODES: &[&str] = &[
		"/dns4/g1.pangolin-p2p.darwinia.network/tcp/10000/p2p/12D3KooWALrn86eyZJikWQhPWsbfHMHnXK1g6FLsGY5voCDKm2AN",
		"/dns4/g2.pangolin-p2p.darwinia.network/tcp/10000/p2p/12D3KooWGwYxLgjoDe7m8dNmkTdXbm8PXZQK2RJkFG4nNqMaDCDB"
	];
}

#[cfg(feature = "alpha")]
mod network {
	pub const NAME: &str = "Pangolin Parachain Alpha";
	pub const ID: &str = "pangolin_parachain_alpha";

	pub const BOOT_NODES: &[&str] = &[
		"/dns4/g3.pangolin-p2p.darwinia.network/tcp/10000/p2p/12D3KooWHPr7V9NyW7Pm7BgNwfKTmz94Ft4MFCmWyr4e6EFQqnCj",
		"/dns4/g4.pangolin-p2p.darwinia.network/tcp/10000/p2p/12D3KooWK4LVr99FkLMdjQbuzKnCYm5Z977GmDCzYuwm6A5rTuME"
	];
}

// --- std ---
use std::str::FromStr;
// --- paritytech ---
use sc_service::{ChainType, GenericChainSpec, Properties};
use sc_telemetry::TelemetryEndpoints;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::UncheckedInto, sr25519};
// --- darwinia-network ---
use super::*;
use network::*;
use pangolin_parachain_runtime::*;

/// Specialized `ChainSpec` for the `Darwinia Parachain` parachain runtime.
pub type ChainSpec = GenericChainSpec<GenesisConfig, Extensions>;

pub const PARA_ID: u32 = 2105;

const TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
fn session_keys(keys: AuraId) -> SessionKeys {
	SessionKeys { aura: keys }
}

fn properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 18.into());
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("tokenSymbol".into(), "PRING".into());

	properties
}

pub fn config() -> Result<ChainSpec, String> {
	#[cfg(feature = "alpha")]
	return ChainSpec::from_json_bytes(
		&include_bytes!("../../res/pangolin-parachain-alpha.json")[..],
	);
	#[cfg(not(feature = "alpha"))]
	return ChainSpec::from_json_bytes(&include_bytes!("../../res/pangolin-parachain.json")[..]);
}

pub fn genesis_config() -> ChainSpec {
	fn genesis() -> GenesisConfig {
		let root: AccountId = array_bytes::hex_into_unchecked(
			"0x72819fbc1b93196fa230243947c1726cbea7e33044c7eb6f736ff345561f9e4c",
		);
		let invulnerables = [
			"0x9c43c00407c0a51e0d88ede9d531f165e370013b648e6b62f4b3bcff4689df02",
			"0x741a9f507722713ec0a5df1558ac375f62469b61d1f60fa60f5dedfc85425b2e",
		]
		.iter()
		.map(|hex| {
			(
				array_bytes::hex_into_unchecked(hex),
				array_bytes::hex2array_unchecked(hex).unchecked_into(),
			)
		})
		.collect::<Vec<_>>();

		GenesisConfig {
			system: SystemConfig { code: wasm_binary_unwrap().into() },
			balances: BalancesConfig { balances: vec![(root.clone(), 100_000 * COIN)] },
			parachain_info: ParachainInfoConfig { parachain_id: PARA_ID.into() },
			collator_selection: CollatorSelectionConfig {
				invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
				candidacy_bond: 0,
				..Default::default()
			},
			session: SessionConfig {
				keys: invulnerables
					.into_iter()
					.map(|(acc, aura)| {
						(
							acc.clone(),        // account id
							acc,                // validator id
							session_keys(aura), // session keys
						)
					})
					.collect(),
			},
			// no need to pass anything to aura, in fact it will panic if we do. Session will take
			// care of this.
			aura: Default::default(),
			aura_ext: Default::default(),
			polkadot_xcm: PolkadotXcmConfig { safe_xcm_version: Some(SAFE_XCM_VERSION) },
			sudo: SudoConfig { key: Some(root) },
			parachain_system: Default::default(),
		}
	}

	return ChainSpec::from_genesis(
		NAME,
		ID,
		ChainType::Live,
		genesis,
		BOOT_NODES.iter().filter_map(|s| FromStr::from_str(s).ok()).collect(),
		Some(
			TelemetryEndpoints::new(vec![(TELEMETRY_URL.to_string(), 0)])
				.expect("Pangolin Parachain telemetry url is valid; qed"),
		),
		None,
		None,
		Some(properties()),
		Extensions { relay_chain: "rococo".into(), para_id: PARA_ID },
	);
}

pub fn development_config() -> ChainSpec {
	fn genesis() -> GenesisConfig {
		let root = get_account_id_from_seed::<sr25519::Public>("Alice");
		let invulnerables = vec![(
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_collator_keys_from_seed("Alice"),
		)];

		GenesisConfig {
			system: SystemConfig { code: wasm_binary_unwrap().into() },
			balances: BalancesConfig {
				balances: invulnerables
					.iter()
					.cloned()
					.map(|(acc, _)| (acc, 100_000 * COIN))
					.collect(),
			},
			parachain_info: ParachainInfoConfig { parachain_id: PARA_ID.into() },
			collator_selection: CollatorSelectionConfig {
				invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
				candidacy_bond: 0,
				..Default::default()
			},
			session: SessionConfig {
				keys: invulnerables
					.into_iter()
					.map(|(acc, aura)| {
						(
							acc.clone(),        // account id
							acc,                // validator id
							session_keys(aura), // session keys
						)
					})
					.collect(),
			},
			// no need to pass anything to aura, in fact it will panic if we do. Session will take
			// care of this.
			aura: Default::default(),
			aura_ext: Default::default(),
			polkadot_xcm: PolkadotXcmConfig { safe_xcm_version: Some(SAFE_XCM_VERSION) },
			sudo: SudoConfig { key: Some(root) },
			parachain_system: Default::default(),
		}
	}

	ChainSpec::from_genesis(
		NAME,
		ID,
		ChainType::Development,
		genesis,
		vec![],
		None,
		None,
		None,
		Some(properties()),
		Extensions { relay_chain: "rococo-local".into(), para_id: PARA_ID },
	)
}
