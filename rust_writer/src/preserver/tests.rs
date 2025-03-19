// SPDX-License-Identifier: GPL-3.0

use super::*;
use std::io::ErrorKind;
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
			preserver2.add_inners(&["fn trait_method"]);
			let preserver3 = Preserver::new("fn main");

			assert_eq!(
				preserved_code,
				apply_preservers(&code, &[&preserver1, &preserver2, &preserver3])
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
			preserver2.add_inners(&["fn trait_method"]);
			let preserver3 = Preserver::new("fn main");

			assert_eq!(
				*preserved_ast,
				preserve_and_parse(
					builder.tempfile_path("complete_file.rs").expect("This exists; qed;"),
					&[&preserver1, &preserver2, &preserver3]
				)
				.expect("This should be Ok; qed;")
			);
		});
}

#[test]
fn preserve_and_parse_fails_if_path_not_readable() {
	TestBuilder::default()
		.with_complete_file()
		.with_read_only_temp_dir()
		.execute(|builder| {
			assert!(matches!(
				preserve_and_parse(
					builder.tempfile_path("complete_file.rs").expect("This exists; qed;"),
					&[]
				), Err(Error::IO(err)) if err.kind() == ErrorKind::PermissionDenied ));
		});
}

#[test]
fn preserve_and_parse_fails_if_non_preservable_code() {
	TestBuilder::default().with_non_preservable_file().execute(|builder| {
		let preserver1 = Preserver::new("struct MyStruct");
		let mut preserver2 = Preserver::new("impl MyTrait for MyStruct");
		preserver2.add_inners(&["fn trait_method"]);
		let preserver3 = Preserver::new("fn main");

		assert!(matches!(
			preserve_and_parse(
				builder.tempfile_path("non_preservable_file.rs").expect("This exists; qed;"),
				&[&preserver1, &preserver2, &preserver3]
			),
			Err(Error::NonPreservableCode)
		));
	});
}

#[test]
fn resolve_preserved_works() {
	TestBuilder::default()
		.with_complete_file()
		.with_resolved_file()
		.with_preserved_file_ast()
		.execute(|builder| {
			let complete_file_path =
				builder.tempfile_path("complete_file.rs").expect("This exists; qed;");

			let resolved_file_path =
				builder.tempfile_path("resolved_file.rs").expect("This exists; qed;");

			let expected_code =
				std::fs::read_to_string(&resolved_file_path).expect("File should be readable");

			assert!(resolve_preserved(
				builder.get_ref_ast_file("preserved_file.rs").expect("This exists; qed;"),
				complete_file_path
			)
			.is_ok());

			let actual_code =
				std::fs::read_to_string(complete_file_path).expect("File should be readable");

			assert_eq!(actual_code, expected_code);
		});
}

#[test]
fn resolve_preserved_fails_if_path_not_writable() {
	TestBuilder::default()
		.with_resolved_file()
		.with_preserved_file_ast()
		.with_read_only_temp_dir()
		.execute(|builder| {
			assert!(matches!(
				resolve_preserved(
					builder.get_ref_ast_file("preserved_file.rs").expect("This exists; qed;"),
					builder.tempfile_path("resolved_file.rs").expect("This exists; qed")
				), Err(Error::IO(err)) if err.kind() == ErrorKind::PermissionDenied ));
		});
}
