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

// --- std ---
use std::marker::PhantomData;
// --- crates ---
use serde::{Deserialize, Serialize};
// --- substrate ---
use cumulus_primitives_core::ParaId;
// --- substrate ---
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
// --- darwinia ---
use darwinia_ethereum_relay::DagsMerkleRootsLoader as DagsMerkleRootsLoaderR;
use darwinia_pc2_primitives::{AccountId, Balance, Signature};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<darwinia_pc2_runtime::GenesisConfig, Extensions>;

type AccountPublic = <Signature as Verify>::Signer;

const DARWINIA_PC2_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

const A_FEW_COINS: Balance = 1 << 44;
const MANY_COINS: Balance = A_FEW_COINS << 6;
const BUNCH_OF_COINS: Balance = MANY_COINS << 6;

const TOKEN_REDEEM_ADDRESS: &'static str = "0x49262B932E439271d05634c32978294C7Ea15d0C";
const DEPOSIT_REDEEM_ADDRESS: &'static str = "0x6EF538314829EfA8386Fc43386cB13B4e0A67D1e";
const SET_AUTHORITIES_ADDRESS: &'static str = "0xD35Bb6F1bc1C84b53E0995c1830454AB7C4147f1";
const RING_TOKEN_ADDRESS: &'static str = "0xb52FBE2B925ab79a821b261C82c5Ba0814AAA5e0";
const KTON_TOKEN_ADDRESS: &'static str = "0x1994100c58753793D52c6f457f189aa3ce9cEe94";
const ETHEREUM_RELAY_AUTHORITY_SIGNER: &'static str = "0x68898db1012808808c903f390909c52d9f706749";

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

	#[cfg(not(feature = "crab"))]
	properties.insert("ss58Format".into(), 18.into());
	#[cfg(feature = "crab")]
	properties.insert("ss58Format".into(), 42.into());
	properties.insert("tokenDecimals".into(), vec![9, 9].into());
	#[cfg(not(feature = "crab"))]
	properties.insert("tokenSymbol".into(), vec!["RING", "KTON"].into());
	#[cfg(feature = "crab")]
	properties.insert("tokenSymbol".into(), vec!["CRING", "CKTON"].into());

	properties
}

pub fn darwinia_pc2_build_spec_config_of(id: ParaId) -> ChainSpec {
	#[cfg(not(feature = "crab"))]
	return ChainSpec::from_genesis(
		"Darwinia PC2",
		"Darwinia PC2",
		ChainType::Live,
		move || darwinia_pc2_build_spec_genesis(id),
		vec![],
		Some(
			TelemetryEndpoints::new(vec![(DARWINIA_PC2_TELEMETRY_URL.to_string(), 0)])
				.expect("Darwinia PC2 telemetry url is valid; qed"),
		),
		// None,
		None,
		Some(properties()),
		// None,
		Extensions {
			relay_chain: "rococo".into(),
			para_id: id.into(),
		},
	);
	#[cfg(feature = "crab")]
	return ChainSpec::from_genesis(
		"Darwinia Crab PC2",
		"Darwinia Crab PC2",
		ChainType::Live,
		move || darwinia_pc2_build_spec_genesis(id),
		vec![],
		Some(
			TelemetryEndpoints::new(vec![(DARWINIA_PC2_TELEMETRY_URL.to_string(), 0)])
				.expect("Darwinia Crab PC2 telemetry url is valid; qed"),
		),
		// None,
		None,
		Some(properties()),
		// None,
		Extensions {
			relay_chain: "rococo".into(),
			para_id: id.into(),
		},
	);
}

fn darwinia_pc2_build_spec_genesis(id: ParaId) -> darwinia_pc2_runtime::GenesisConfig {
	let root = AccountId::from(array_bytes::hex2array_unchecked!(
		"0x72819fbc1b93196fa230243947c1726cbea7e33044c7eb6f736ff345561f9e4c",
		32
	));
	let endowed_accounts = vec![(root.clone(), BUNCH_OF_COINS)];
	let collective_members = vec![get_account_id_from_seed::<sr25519::Public>("Alice")];

	darwinia_pc2_runtime::GenesisConfig {
		frame_system: darwinia_pc2_runtime::SystemConfig {
			code: darwinia_pc2_runtime::wasm_binary_unwrap().into(),
			changes_trie_config: Default::default(),
		},
		darwinia_balances_Instance0: darwinia_pc2_runtime::BalancesConfig {
			balances: endowed_accounts,
		},
		darwinia_balances_Instance1: Default::default(),
		darwinia_democracy: Default::default(),
		pallet_collective_Instance0: darwinia_pc2_runtime::CouncilConfig {
			phantom: PhantomData::<darwinia_pc2_runtime::CouncilCollective>,
			members: collective_members.clone(),
		},
		pallet_collective_Instance1: darwinia_pc2_runtime::TechnicalCommitteeConfig {
			phantom: PhantomData::<darwinia_pc2_runtime::TechnicalCollective>,
			members: collective_members
		},
		darwinia_elections_phragmen: Default::default(),
		pallet_membership_Instance0: Default::default(),
		pallet_sudo: darwinia_pc2_runtime::SudoConfig { key: root },
		darwinia_ethereum_relay: darwinia_pc2_runtime::EthereumRelayConfig {
			genesis_header_info: (
				vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 29, 204, 77, 232, 222, 199, 93, 122, 171, 133, 181, 103, 182, 204, 212, 26, 211, 18, 69, 27, 148, 138, 116, 19, 240, 161, 66, 253, 64, 212, 147, 71, 128, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 33, 123, 11, 188, 251, 114, 226, 213, 126, 40, 243, 60, 179, 97, 185, 152, 53, 19, 23, 119, 85, 220, 63, 51, 206, 62, 112, 34, 237, 98, 183, 123, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 132, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 136, 0, 0, 0, 0, 0, 0, 0, 66, 1, 65, 148, 16, 35, 104, 9, 35, 224, 254, 77, 116, 163, 75, 218, 200, 20, 31, 37, 64, 227, 174, 144, 98, 55, 24, 228, 125, 102, 209, 202, 74, 45],
				b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".into()
			),
			dags_merkle_roots_loader: DagsMerkleRootsLoaderR::from_file(
				"res/ethereum/dags-merkle-roots.json",
				"DAG_MERKLE_ROOTS_PATH",
			),
			..Default::default()
		},
		darwinia_ethereum_backing: darwinia_pc2_runtime::EthereumBackingConfig {
			token_redeem_address: array_bytes::hex2array_unchecked!(TOKEN_REDEEM_ADDRESS, 20).into(),
			deposit_redeem_address: array_bytes::hex2array_unchecked!(DEPOSIT_REDEEM_ADDRESS, 20).into(),
			set_authorities_address: array_bytes::hex2array_unchecked!(SET_AUTHORITIES_ADDRESS, 20).into(),
			ring_token_address: array_bytes::hex2array_unchecked!(RING_TOKEN_ADDRESS, 20).into(),
			kton_token_address: array_bytes::hex2array_unchecked!(KTON_TOKEN_ADDRESS, 20).into(),
			ring_locked: BUNCH_OF_COINS,
			kton_locked: BUNCH_OF_COINS,
		},
		darwinia_relay_authorities_Instance0: darwinia_pc2_runtime::EthereumRelayAuthoritiesConfig {
			authorities: vec![(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				array_bytes::hex2array_unchecked!(ETHEREUM_RELAY_AUTHORITY_SIGNER, 20).into(),
				1
			)]
		},
		parachain_info: darwinia_pc2_runtime::ParachainInfoConfig { parachain_id: id },
	}
}

pub fn darwinia_pc2_development_config_of(id: ParaId) -> ChainSpec {
	#[cfg(not(feature = "crab"))]
	return ChainSpec::from_genesis(
		"Darwinia PC2",
		"Darwinia PC2",
		ChainType::Development,
		move || darwinia_pc2_development_genesis(id),
		vec![],
		None,
		None,
		Some(properties()),
		// None,
		Extensions {
			relay_chain: "rococo".into(),
			para_id: id.into(),
		},
	);
	#[cfg(feature = "crab")]
	return ChainSpec::from_genesis(
		"Darwinia Crab PC2",
		"Darwinia Crab PC2",
		ChainType::Development,
		move || darwinia_pc2_development_genesis(id),
		vec![],
		None,
		None,
		Some(properties()),
		// None,
		Extensions {
			relay_chain: "rococo".into(),
			para_id: id.into(),
		},
	);
}

fn darwinia_pc2_development_genesis(id: ParaId) -> darwinia_pc2_runtime::GenesisConfig {
	let root = get_account_id_from_seed::<sr25519::Public>("Alice");
	let endowed_accounts = vec![
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		get_account_id_from_seed::<sr25519::Public>("Bob"),
		get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
		get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
	];
	let collective_members = vec![get_account_id_from_seed::<sr25519::Public>("Alice")];

	darwinia_pc2_runtime::GenesisConfig {
		frame_system: darwinia_pc2_runtime::SystemConfig {
			code: darwinia_pc2_runtime::wasm_binary_unwrap().into(),
			changes_trie_config: Default::default(),
		},
		darwinia_balances_Instance0: darwinia_pc2_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, BUNCH_OF_COINS))
				.collect(),
		},
		darwinia_balances_Instance1: darwinia_pc2_runtime::KtonConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, BUNCH_OF_COINS))
				.collect(),
		},
		darwinia_democracy: Default::default(),
		pallet_collective_Instance0: darwinia_pc2_runtime::CouncilConfig {
			phantom: PhantomData::<darwinia_pc2_runtime::CouncilCollective>,
			members: collective_members.clone(),
		},
		pallet_collective_Instance1: darwinia_pc2_runtime::TechnicalCommitteeConfig {
			phantom: PhantomData::<darwinia_pc2_runtime::TechnicalCollective>,
			members: collective_members
		},
		darwinia_elections_phragmen: Default::default(),
		pallet_membership_Instance0: Default::default(),
		pallet_sudo: darwinia_pc2_runtime::SudoConfig { key: root },
		darwinia_ethereum_relay: darwinia_pc2_runtime::EthereumRelayConfig {
			genesis_header_info: (
				vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 29, 204, 77, 232, 222, 199, 93, 122, 171, 133, 181, 103, 182, 204, 212, 26, 211, 18, 69, 27, 148, 138, 116, 19, 240, 161, 66, 253, 64, 212, 147, 71, 128, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 53, 33, 123, 11, 188, 251, 114, 226, 213, 126, 40, 243, 60, 179, 97, 185, 152, 53, 19, 23, 119, 85, 220, 63, 51, 206, 62, 112, 34, 237, 98, 183, 123, 86, 232, 31, 23, 27, 204, 85, 166, 255, 131, 69, 230, 146, 192, 248, 110, 91, 72, 224, 27, 153, 108, 173, 192, 1, 98, 47, 181, 227, 99, 180, 33, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 132, 160, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 36, 136, 0, 0, 0, 0, 0, 0, 0, 66, 1, 65, 148, 16, 35, 104, 9, 35, 224, 254, 77, 116, 163, 75, 218, 200, 20, 31, 37, 64, 227, 174, 144, 98, 55, 24, 228, 125, 102, 209, 202, 74, 45],
				b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".into()
			),
			dags_merkle_roots_loader: DagsMerkleRootsLoaderR::from_file(
				"res/ethereum/dags-merkle-roots.json",
				"DAG_MERKLE_ROOTS_PATH",
			),
			..Default::default()
		},
		darwinia_ethereum_backing: darwinia_pc2_runtime::EthereumBackingConfig {
			token_redeem_address: array_bytes::hex2array_unchecked!(TOKEN_REDEEM_ADDRESS, 20).into(),
			deposit_redeem_address: array_bytes::hex2array_unchecked!(DEPOSIT_REDEEM_ADDRESS, 20).into(),
			set_authorities_address: array_bytes::hex2array_unchecked!(SET_AUTHORITIES_ADDRESS, 20).into(),
			ring_token_address: array_bytes::hex2array_unchecked!(RING_TOKEN_ADDRESS, 20).into(),
			kton_token_address: array_bytes::hex2array_unchecked!(KTON_TOKEN_ADDRESS, 20).into(),
			ring_locked: BUNCH_OF_COINS,
			kton_locked: BUNCH_OF_COINS,
		},
		darwinia_relay_authorities_Instance0: darwinia_pc2_runtime::EthereumRelayAuthoritiesConfig {
			authorities: vec![(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				array_bytes::hex2array_unchecked!(ETHEREUM_RELAY_AUTHORITY_SIGNER, 20).into(),
				1
			)]
		},
		parachain_info: darwinia_pc2_runtime::ParachainInfoConfig { parachain_id: id },
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
