// SPDX-License-Identifier: GPL-3.0

use super::*;

#[test]
fn create_preserver() {
	let preserver = Preserver::new("root");
	assert_eq!(preserver.lookup(), "root");
	assert!(preserver.inner.is_none());
}

#[test]
fn add_inners_to_preserver() {
	let mut preserver = Preserver::new("root");
	preserver.add_inners(&["inner1", "inner2", "inner3"]);

	assert_eq!(preserver.lookup(), "root");
	let mut inner = preserver.get_inner().expect("No inner");
	assert_eq!(inner.lookup(), "inner1");
	inner = inner.get_inner().expect("No inner");
	assert_eq!(inner.lookup(), "inner2");
	inner = inner.get_inner().expect("No inner");
	assert_eq!(inner.lookup(), "inner3");
	assert!(inner.get_inner().is_none());
}

#[test]
fn preserver_should_handle_empty_inners() {
	let mut preserver = Preserver::new("root");
	preserver.add_inners(&[]);
	assert_eq!(preserver.lookup(), "root");
	assert!(preserver.inner.is_none());
}

#[test]
fn create_delimiters_count() {
	let delimiters_count = DelimitersCount::new();
	assert!(delimiters_count.is_complete());
}

#[test]
fn count_delimiters() {
	let mut delimiters_count = DelimitersCount::new();

	delimiters_count.count("{ ( [ ) ] }");
	assert_eq!(delimiters_count.counts[0], 1); // '{'
	assert_eq!(delimiters_count.counts[1], 1); // '}'
	assert_eq!(delimiters_count.counts[2], 1); // '('
	assert_eq!(delimiters_count.counts[3], 1); // ')'
	assert_eq!(delimiters_count.counts[4], 1); // '['
	assert_eq!(delimiters_count.counts[5], 1); // ']'
}

#[test]
fn delimiters_count_should_be_complete() {
	let mut delimiters_count = DelimitersCount::new();

	delimiters_count.count("{ ( [ ) ] }");
	assert!(delimiters_count.is_complete());
}

#[test]
fn delimiters_count_should_not_be_complete() {
	let mut delimiters_count = DelimitersCount::new();

	delimiters_count.count("{ ( [ ) }");
	assert!(!delimiters_count.is_complete());
}
