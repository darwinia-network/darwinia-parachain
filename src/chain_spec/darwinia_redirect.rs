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

// --- paritytech ---
use sc_service::{ChainType, GenericChainSpec, Properties};
use sc_telemetry::TelemetryEndpoints;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::UncheckedInto, sr25519};
// --- darwinia-network ---
use super::*;
use darwinia_redirect_runtime::*;

/// Specialized `ChainSpec` for the `Darwinia Redirect` parachain runtime.
pub type ChainSpec = GenericChainSpec<GenesisConfig, Extensions>;

pub const PARA_ID: u32 = 2003;

const TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
fn session_keys(keys: AuraId) -> SessionKeys {
	SessionKeys { aura: keys }
}

fn properties() -> Properties {
	let mut properties = Properties::new();

	properties.insert("ss58Format".into(), 42.into());
	properties.insert("tokenDecimals".into(), 9.into());
	properties.insert("tokenSymbol".into(), "RING".into());

	properties
}

pub fn config() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../res/darwinia-redirect.json")[..])
}

pub fn genesis_config() -> ChainSpec {
	fn genesis() -> GenesisConfig {
		let root: AccountId = array_bytes::hex_into_unchecked(
			"0x129d025b24257aabdefac93d00419f06a38e3a5e2314dd6866b16e8f205ce074",
		);
		let invulnerables = [
			// Denny
			"0x7e8672b2c2ad0904ba6137de480eaa3b9476042f3f2ae08da033c4ccf2272d5a",
			"0xbe7e6c55feca7ffbfd961c93acdf1bc68bea91d758fb8da92f65c66bbf12ea74",
			// Way
			"0xea0f4185dd32c1278d7bbd3cdd2fbaec3ca29921a88c04c175401a0668d88e66",
			"0x56695000227fee2b4e2b15e892527250e47d4671e17f6e604cd67fb7213bbc19",
			// Xavier
			"0xb4f7f03bebc56ebe96bc52ea5ed3159d45a0ce3a8d7f082983c33ef133274747",
			"0x28b4a5e67767ec4aba8e8d99ac58481ec74e48185507f1552b1f8ba00994cf59",
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
			system: SystemConfig {
				code: wasm_binary_unwrap().into(),
				changes_trie_config: Default::default(),
			},
			balances: BalancesConfig {
				balances: vec![
					// Root
					(root.clone(), 100_000 * COIN),
					// Denny
					(
						array_bytes::hex_into_unchecked(
							"0x0a66532a23c418cca12183fee5f6afece770a0bb8725f459d7d1b1b598f91c49",
						),
						100_000 * COIN,
					),
				],
			},
			parachain_info: ParachainInfoConfig {
				parachain_id: PARA_ID.into(),
			},
			collator_selection: CollatorSelectionConfig {
				invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
				candidacy_bond: 0,
				..Default::default()
			},
			session: SessionConfig {
				keys: invulnerables
					.iter()
					.cloned()
					.map(|(acc, aura)| {
						(
							acc.clone(),        // account id
							acc.clone(),        // validator id
							session_keys(aura), // session keys
						)
					})
					.collect(),
			},
			// no need to pass anything to aura, in fact it will panic if we do. Session will take care
			// of this.
			aura: Default::default(),
			aura_ext: Default::default(),
			sudo: SudoConfig { key: root },
			parachain_system: Default::default(),
		}
	}

	return ChainSpec::from_genesis(
		"Darwinia Redirect",
		"Darwinia Redirect",
		ChainType::Live,
		genesis,
		vec![],
		Some(
			TelemetryEndpoints::new(vec![(TELEMETRY_URL.to_string(), 0)])
				.expect("Darwinia Redirect telemetry url is valid; qed"),
		),
		None,
		Some(properties()),
		Extensions {
			relay_chain: "polkadot".into(),
			para_id: PARA_ID,
		},
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
			system: SystemConfig {
				code: wasm_binary_unwrap().into(),
				changes_trie_config: Default::default(),
			},
			balances: BalancesConfig {
				balances: invulnerables
					.iter()
					.cloned()
					.map(|(acc, _)| (acc, 100_000 * COIN))
					.collect(),
			},
			parachain_info: ParachainInfoConfig {
				parachain_id: PARA_ID.into(),
			},
			collator_selection: CollatorSelectionConfig {
				invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
				candidacy_bond: 0,
				..Default::default()
			},
			session: SessionConfig {
				keys: invulnerables
					.iter()
					.cloned()
					.map(|(acc, aura)| {
						(
							acc.clone(),        // account id
							acc.clone(),        // validator id
							session_keys(aura), // session keys
						)
					})
					.collect(),
			},
			// no need to pass anything to aura, in fact it will panic if we do. Session will take care
			// of this.
			aura: Default::default(),
			aura_ext: Default::default(),
			sudo: SudoConfig { key: root },
			parachain_system: Default::default(),
		}
	}

	return ChainSpec::from_genesis(
		"Darwinia Redirect Dev",
		"Darwinia Redirect Dev",
		ChainType::Development,
		genesis,
		vec![],
		None,
		None,
		Some(properties()),
		Extensions {
			relay_chain: "rococo-local".into(),
			para_id: PARA_ID,
		},
	);
}
