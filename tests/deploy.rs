#!/bin/bash
#![allow()] /*
			set -e
			rustc $0 -o ${0/.rs/.bin} -g
			exec ${0/.rs/.bin} $@
			*/

use std::{env, path::PathBuf};

fn main() {
	if let Some(deployer) = Deployer::new() {
		deployer.deploy();
	}
}

struct Deployer {
	chain: String,
	project_dir: PathBuf,
	darwinia_exec: PathBuf,
	polkadot_exec: PathBuf,
}
impl Deployer {
	fn new() -> Option<Self> {
		let chain = if let Some(c) = Self::get_chain() {
			c
		} else {
			return None;
		};
		let project_dir = Self::get_project_dir();
		let (darwinia_exec, polkadot_exec) = if let Some(p) = Self::get_execs(&project_dir) {
			p
		} else {
			return None;
		};

		Some(Self {
			chain,
			project_dir,
			darwinia_exec,
			polkadot_exec,
		})
	}

	fn get_chain() -> Option<String> {
		let args = env::args().collect::<Vec<_>>();

		if let Some(chain) = args.get(1) {
			println!("[CHECK] chain: {}", chain);

			Some(chain.to_owned())
		} else {
			println!("[ERROR] usage: deploy.rs <CHAIN>");

			None
		}
	}

	fn get_project_dir() -> PathBuf {
		let script_path = env::current_exe().unwrap();
		let project_dir = script_path.parent().unwrap().parent().unwrap().to_owned();

		println!(
			"[CHECK] project directory: {}",
			project_dir.to_string_lossy()
		);

		project_dir
	}

	fn get_execs(project_dir: &PathBuf) -> Option<(PathBuf, PathBuf)> {
		fn get_darwinia_exec(project_dir: &PathBuf) -> Option<PathBuf> {
			let target_1 = project_dir.join("target/release/darwinia-collator");

			if target_1.is_file() {
				println!("[CHECK] executable: {}", target_1.to_string_lossy());

				return Some(target_1);
			}

			let target_2 = project_dir.join("tests/bin/darwinia-collator");

			if target_2.is_file() {
				println!("[CHECK] executable: {}", target_2.to_string_lossy());

				return Some(target_2);
			}

			println!(
				"[WARN] darwinia-collator executable not found, searched:\
				\n       {}\
				\n       {}",
				target_1.to_string_lossy(),
				target_2.to_string_lossy()
			);

			return None;
		}

		fn get_polkadot_exec(project_dir: &PathBuf) -> Option<(PathBuf)> {
			let target = project_dir.join("tests/bin/polkadot");

			if target.is_file() {
				println!("[CHECK] executable: {}", target.to_string_lossy());

				return Some(target);
			}

			println!(
				"[WARN] polkadot executable not found, searched:\
				\n       {}",
				target.to_string_lossy()
			);

			return None;
		}

		Some((
			get_darwinia_exec(project_dir)?,
			get_polkadot_exec(project_dir)?,
		))
	}

	fn deploy(self) {}
}
