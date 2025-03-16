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

#[test]
fn preserve_and_parse_works() {
	TestBuilder::default()
		.with_complete_file()
		.with_preserved_file_ast()
		.execute(|builder| {
			let preserved_ast =
				builder.get_ref_ast_file("preserved_file.rs").expect("This exists; qed;");

			let preserver1 = Preserver::new("struct MyStruct");
			let mut preserver2 = Preserver::new("impl MyTrait for MyStruct");
			preserver2.add_inners(vec!["fn trait_method"]);
			let preserver3 = Preserver::new("fn main");

			assert_eq!(
				*preserved_ast,
				preserve_and_parse(
					builder.tempfile_path("complete_file.rs").expect("This exists; qed;"),
					vec![preserver1, preserver2, preserver3]
				)
				.expect("This should be Ok; qed;")
			);
		});
}
