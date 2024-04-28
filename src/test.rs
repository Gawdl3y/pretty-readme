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
