
/// Namespace for operations that cannot be added to any other modules.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Miscellaneous {}

impl Miscellaneous {
    #[inline]
    pub fn get() -> MiscellaneousGetBuilder {
        MiscellaneousGetBuilder
    }
}

/// Builder created by [`Miscellaneous::get`](./struct.Miscellaneous.html#method.get) method for a `GET` operation associated with `Miscellaneous`.
#[derive(Debug, Clone)]
pub struct MiscellaneousGetBuilder;


impl<Client: crate::client::ApiClient + Sync + 'static> crate::client::Sendable<Client> for MiscellaneousGetBuilder {
    type Output = String;

    const METHOD: http::Method = http::Method::GET;

    fn rel_path(&self) -> std::borrow::Cow<'static, str> {
        "/ping".into()
    }
}
