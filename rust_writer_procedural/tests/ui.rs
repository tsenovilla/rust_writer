// SPDX-License-Identifier: GPL-3.0

use trybuild::TestCases;

#[test]
fn ui() {
	let t = TestCases::new();
	t.compile_fail("tests/ui/compile_fail/*.rs");
	t.pass("tests/ui/pass/*.rs");
}
