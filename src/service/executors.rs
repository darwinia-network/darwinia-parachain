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

// --- paritytech ---
use sc_executor::{NativeExecutionDispatch, NativeVersion};
use sp_io::SubstrateHostFunctions;

#[cfg(not(feature = "runtime-benchmarks"))]
pub type HostFunctions = SubstrateHostFunctions;
#[cfg(feature = "runtime-benchmarks")]
pub type HostFunctions = (SubstrateHostFunctions, frame_benchmarking::benchmarking::HostFunctions);

/// Darwinia native executor instance.
pub struct DarwiniaParachainRuntimeExecutor;
impl NativeExecutionDispatch for DarwiniaParachainRuntimeExecutor {
	#[cfg(feature = "runtime-benchmarks")]
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type ExtendHostFunctions = ();

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		darwinia_parachain_runtime::api::dispatch(method, data)
	}

	fn native_version() -> NativeVersion {
		darwinia_parachain_runtime::native_version()
	}
}

/// Crab Parachain native executor instance.
pub struct CrabParachainRuntimeExecutor;
impl NativeExecutionDispatch for CrabParachainRuntimeExecutor {
	#[cfg(feature = "runtime-benchmarks")]
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type ExtendHostFunctions = ();

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		crab_parachain_runtime::api::dispatch(method, data)
	}

	fn native_version() -> NativeVersion {
		crab_parachain_runtime::native_version()
	}
}

/// Pangolin Parachain native executor instance.
pub struct PangolinParachainRuntimeExecutor;
impl NativeExecutionDispatch for PangolinParachainRuntimeExecutor {
	#[cfg(feature = "runtime-benchmarks")]
	type ExtendHostFunctions = frame_benchmarking::benchmarking::HostFunctions;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type ExtendHostFunctions = ();

	fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
		pangolin_parachain_runtime::api::dispatch(method, data)
	}

	fn native_version() -> NativeVersion {
		pangolin_parachain_runtime::native_version()
	}
}
