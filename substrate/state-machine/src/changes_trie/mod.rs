// Copyright 2017 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Changes trie related structures and functions.
//!
//! Changes trie is a trie built of { storage key => extrinsiscs } pairs
//! at the end of each block. For every changed storage key it contains
//! a pair, mapping key to the set of extrinsics where it has been changed.
//!
//! Optionally, every N blocks, additional level1-digest nodes are appended
//! to the changes trie, containing pairs { storage key => blocks }. For every
//! storage key that has been changed in PREVIOUS N-1 blocks (except for genesis
//! block) it contains a pair, mapping this key to the set of blocks where it
//! has been changed.
//!
//! Optionally, every N^digest_level (where digest_level > 1) blocks, additional
//! digest_level digest is created. It is built out of pairs { storage key => digest
//! block }, containing entries for every storage key that has been changed in
//! the last N*digest_level-1 blocks (except for genesis block), mapping these keys
//! to the set of lower-level digest blocks.

mod build;
mod build_iterator;
mod changes_iterator;
mod input;
mod storage;

pub use self::storage::InMemoryStorage;

use std::sync::Arc;
use trie_backend::{DBValue, TrieH256};
use changes_trie::build::prepare_input;
use overlayed_changes::OverlayedChanges;
use {Storage as TrieStorage};

/// Changes trie storage. Provides access to trie roots and trie nodes.
pub trait Storage: Send + Sync {
	/// Get changes trie root for given block.
	fn root(&self, block: u64) -> Result<Option<TrieH256>, String>;

	/// Get a trie node.
	fn get(&self, key: &TrieH256) -> Result<Option<DBValue>, String>;
}

/// Changes trie configuration.
#[derive(Debug, Clone)]
pub struct Configuration {
	/// Interval (in blocks) at which level1-digests are created. Digests are not
	/// created when this is less or equal to 1.
	pub digest_interval: u64,
	/// Maximal number of digest levels in hierarchy. 0 means that digests are not
	/// created at all (even level1 digests). 1 means only level1-digests are created.
	/// 2 means that every digest_interval^2 there will be a level2-digest, and so on.
	pub digest_levels: u8,
}

/// Compute the changes trie root and transaction for given block.
/// Returns None if there's no data to perform computation.
pub fn compute_changes_trie_root(storage: Option<Arc<Storage>>, changes: &OverlayedChanges)
	-> Option<([u8; 32], Vec<(Vec<u8>, Vec<u8>)>)>
{
	let input_pairs = prepare_input(storage, changes)?;
	let transaction = input_pairs.into_iter()
		.map(Into::into)
		.collect::<Vec<_>>();
	let root = ::triehash::trie_root(transaction.iter().map(|(k, v)| (&*k, &*v))).0;

	Some((root, transaction))
}
