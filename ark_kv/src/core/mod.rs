/// Core errors.
#[derive(Debug, Fail)]
pub enum Error {
    /// TODO(refactor): Clean up errors.
    #[fail(display = "CoreError::Unwrap")]
    Unwrap,
}
