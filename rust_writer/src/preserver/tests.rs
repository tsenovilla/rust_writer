// SPDX-License-Identifier: GPL-3.0

use super::*;
use test_builder::TestBuilder;

#[test]
fn apply_preservers_works() {
	TestBuilder::default()
		.with_complete_file()
		.with_preserved_file()
		.execute(|builder| {
			let code = std::fs::read_to_string(
				builder.tempfile_path("complete_file.rs").expect("This exists; qed;"),
			)
			.expect("File should be readable");

			let preserved_code = std::fs::read_to_string(
				builder.tempfile_path("preserved_file.rs").expect("This exists; qed;"),
			)
			.expect("File should be readable");

			let preserver1 = Preserver::new("struct MyStruct");
			let mut preserver2 = Preserver::new("impl MyTrait for MyStruct");
			preserver2.add_inners(vec!["fn trait_method"]);
			let preserver3 = Preserver::new("fn main");

			assert_eq!(
				preserved_code,
				apply_preservers(code, vec![preserver1, preserver2, preserver3])
			);
		});
}
