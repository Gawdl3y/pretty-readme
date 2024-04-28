# Pretty Readme

[![Crates.io Version](https://img.shields.io/crates/v/pretty-readme)](https://crates.io/crates/pretty-readme)
[![docs.rs](https://img.shields.io/docsrs/pretty-readme)](https://docs.rs/pretty-readme)

Simple crate containing a procedural macro to easily adapt an input readme markdown file for Rustdoc use.
The goal is to make it trivial to write a README.md file that looks and functions (links, examples, etc.) perfectly
on both GitHub (or another repository location) and Docs.rs (or local docs builds).

Allows for running any included Rust codeblock examples as doctests, even if they contain the question mark (`?`)
operator without a specified function or return Result type.
Also replaces a given docs URL with another, which can be used to replace absolute links to docs items with relative
links in the built documentation.

Originally written for the [tyche] crate.

# Examples

Given the input `README.md` file:

````md
# Some Cool Crate!

Some Cool Crate defines the [StuffDoer] type for other things to do stuff with. Wow!

## Examples

```rust
use some_cool_crate::StuffDoer;

// Do some stuff
StuffDoer::do_stuff();

// Do some stuff, but fallibly
StuffDoer::do_stuff_that_might_fail()?;
```

## License

IDK, do whatever you want.

[StuffDoer]: https://docs.rs/super-cool-crate/latest/super-cool-crate/struct.StuffDoer.html
````

With the library's `lib.rs`:

```rust
#![doc = pretty_readme::docify!("README.md", "https://docs.rs/super-cool-crate/latest/super-cool-crate/", "./")]

pub struct StuffDoer;

impl StuffDoer {
	pub fn do_stuff() {
		// we lied, we do nothing here
	}

	pub fn do_stuff_that_might_fail() -> Result<(), SomeError> {
		// we lied again, nothing will ever fail
		Ok(())
	}
}

#[derive(Debug)]
pub struct SomeError;

impl std::fmt::Display for SomeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Some error occurred")
	}
}

impl std::error::Error for SomeError {}
```

This example accomplishes the following:

- Allows the example codeblock in the readme to be run as a doctest without requiring an explicit function or Result
  type even though the question mark operator is used, keeping the example tidy when viewing it on GitHub
- Replaces the full URL to the `StuffDoer` type's docs page with a relative one so that when viewing it on docs.rs
  or a local docs build, it always links to the currently-selected version being viewed

## Contributing

Although this crate was made for a fairly specifiy purpose, contributions are absolutely welcome if you have ideas!
Try to keep PRs relatively small in scope (single feature/fix/refactor at a time) and word your commits descriptively.

## License

Pretty Readme is licensed under the [LGPLv3](https://www.gnu.org/licenses/lgpl-3.0) license.

[tyche]: https://github.com/Gawdl3y/tyche-rs
