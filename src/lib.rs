#![allow(clippy::tabs_in_doc_comments)]

//! Simple crate containing a procedural macro to easily adapt an input readme markdown file for Rustdoc use.
//! The goal is to make it trivial to write a README.md file that looks and functions (links, examples, etc.) perfectly
//! on both GitHub (or another repository location) and Docs.rs (or local docs builds).
//!
//! Allows for running any included Rust codeblock examples as doctests, even if they contain the question mark (`?`)
//! operator without a specified function or return Result type.
//! Also replaces a given docs URL with another, which can be used to replace absolute links to docs items with relative
//! links in the built documentation.
//!
//! Originally written for the [tyche] crate.
//!
//! # Examples
//! Given the input `README.md` file:
//! ````md
//! # Some Cool Crate!
//! Some Cool Crate defines the [StuffDoer] type for other things to do stuff with. Wow!
//!
//! ## Examples
//! ```rust
//! use some_cool_crate::StuffDoer;
//!
//! // Do some stuff
//! StuffDoer::do_stuff();
//!
//! // Do some stuff, but fallibly
//! StuffDoer::do_stuff_that_might_fail()?;
//! ```
//!
//! ## License
//! IDK, do whatever you want.
//!
//! [StuffDoer]: https://docs.rs/super-cool-crate/latest/super-cool-crate/struct.StuffDoer.html
//! ````
//!
//! With the library's `lib.rs`:
//! ```rust
//! #![doc = pretty_readme::docify!("README.md", "https://docs.rs/super-cool-crate/latest/super-cool-crate/", "./")]
//!
//! pub struct StuffDoer;
//!
//! impl StuffDoer {
//! 	pub fn do_stuff() {
//! 		// we lied, we do nothing here
//! 	}
//!
//! 	pub fn do_stuff_that_might_fail() -> Result<(), SomeError> {
//! 		// we lied again, nothing will ever fail
//! 		Ok(())
//! 	}
//! }
//!
//! #[derive(Debug)]
//! pub struct SomeError;
//!
//! impl std::fmt::Display for SomeError {
//! 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//! 		write!(f, "Some error occurred")
//! 	}
//! }
//!
//! impl std::error::Error for SomeError {}
//! ```
//!
//! This example accomplishes the following:
//! - Allows the example codeblock in the readme to be run as a doctest without requiring an explicit function or Result
//!   type even though the question mark operator is used, keeping the example tidy when viewing it on GitHub
//! - Replaces the full URL to the `StuffDoer` type's docs page with a relative one so that when viewing it on docs.rs
//!   or a local docs build, it always links to the currently-selected version being viewed
//!
//! [tyche]: https://github.com/Gawdl3y/tyche-rs

#![deny(macro_use_extern_crate, meta_variable_misuse, unit_bindings)]
#![warn(
	explicit_outlives_requirements,
	missing_docs,
	missing_debug_implementations,
	unreachable_pub,
	unused_crate_dependencies,
	unused_qualifications,
	clippy::pedantic,
	clippy::absolute_paths,
	clippy::arithmetic_side_effects,
	clippy::clone_on_ref_ptr,
	clippy::cognitive_complexity,
	clippy::empty_enum_variants_with_brackets,
	clippy::empty_structs_with_brackets,
	clippy::exhaustive_enums,
	clippy::exhaustive_structs,
	clippy::filetype_is_file,
	clippy::missing_const_for_fn,
	clippy::fn_to_numeric_cast_any,
	clippy::format_push_string,
	clippy::get_unwrap,
	clippy::if_then_some_else_none,
	clippy::lossy_float_literal,
	clippy::map_err_ignore,
	clippy::missing_docs_in_private_items,
	clippy::multiple_inherent_impl,
	clippy::mutex_atomic,
	clippy::panic_in_result_fn,
	clippy::print_stderr,
	clippy::print_stdout,
	clippy::pub_without_shorthand,
	clippy::rc_buffer,
	clippy::rc_mutex,
	clippy::redundant_type_annotations,
	clippy::ref_patterns,
	clippy::rest_pat_in_fully_bound_structs,
	clippy::same_name_method,
	clippy::self_named_module_files,
	clippy::str_to_string,
	clippy::string_to_string,
	clippy::suspicious_xor_used_as_pow,
	clippy::tests_outside_test_module,
	clippy::try_err,
	clippy::undocumented_unsafe_blocks,
	clippy::unnecessary_safety_comment,
	clippy::unnecessary_safety_doc,
	clippy::unnecessary_self_imports,
	clippy::unneeded_field_pattern,
	clippy::unwrap_in_result,
	clippy::verbose_file_reads
)]

use std::{env, fs, path::Path};

use quote::ToTokens;
use regex::RegexBuilder;
use syn::{parse::Parser, punctuated::Punctuated, spanned::Spanned};

/// Type to parse the macro input into
type Args = Punctuated<syn::LitStr, syn::Token![,]>;

/// Takes an input readme file path (relative to Cargo.toml), reads the contents of the file,
/// adds `# Ok::<(), Box<dyn std::error::Error>>(())` to the end of all Rust code blocks inside it,
/// and replaces a given docs URL with the given replacement URL, returning the resulting string as a token.
///
/// See the [crate documentation] for more information.
///
/// # Examples
/// ```
/// #[doc = pretty_readme::docify!("README.md", "https://docs.rs/some_crate/latest/some_crate/", "./")]
/// mod some_module {
/// 	// ...
/// }
/// ```
///
/// [crate documentation]: crate
#[proc_macro]
#[allow(clippy::missing_panics_doc)]
pub fn docify(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
	let input = proc_macro2::TokenStream::from(input);
	let input_span = input.span();

	// Extract each parameter from the input tokens
	let args = match Args::parse_terminated.parse(input.into()) {
		Ok(args) => Vec::from_iter(args),
		Err(err) => return err.into_compile_error().into(),
	};
	let (path, text, replacement) = match args.as_slice() {
		[path, text, replacement] => (path, text.value(), replacement.value()),
		_ => {
			return syn::Error::new(
				input_span,
				r#"expected `"<readme_path>", "<docs_url>", "<replacement_docs_url>"`"#,
			)
			.into_compile_error()
			.into()
		}
	};

	// Resolve the readme path
	let project_root = env::var("CARGO_MANIFEST_DIR").unwrap_or(".".to_owned());
	let readme_path = Path::new(&project_root).join(path.value());

	// Read the contents of the readme
	let readme = if readme_path.is_file() {
		match fs::read_to_string(&readme_path) {
			Ok(contents) => contents,
			Err(err) => {
				return syn::Error::new_spanned(path, format!("Error reading readme file at {readme_path:?}: {err}"))
					.into_compile_error()
					.into()
			}
		}
	} else {
		return syn::Error::new_spanned(
			path,
			format!("Readme file at {readme_path:?} not found; path must be relative to Cargo.toml"),
		)
		.into_compile_error()
		.into();
	};

	// Insert "# Ok::<(), Box<dyn std::error::Error>>(())" at the end of all Rust codeblocks
	let re = RegexBuilder::new(r"```(rust|rs)(\r\n|\r|\n)(.+?)(\r\n|\r|\n)```")
		.dot_matches_new_line(true)
		.case_insensitive(true)
		.build()
		.expect("unable to build codeblock regex");
	let readme = re.replace_all(&readme, "```$1$2$3$4$4# Ok::<(), Box<dyn std::error::Error>>(())$4```");

	// Replace the given docs URL with the given replacement
	readme.replace(&text, &replacement).into_token_stream().into()
}
