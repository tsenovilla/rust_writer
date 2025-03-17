// SPDX-License-Identifier: GPL-3.0

use thiserror::Error;

/// Represents the various errors that can occur in the crate.
#[derive(Error, Debug)]
pub enum Error {
	#[error("IO error: `{0}`")]
	IO(#[from] std::io::Error),
	#[error("{0}")]
	Descriptive(String),
	#[error("The code cannot be safely preserved. Check 'https://docs.rs/rust_writer/latest/rust_writer/preserver/fn.preserve_and_parse.html' for further information.")]
	NonPreservableCode,
}
