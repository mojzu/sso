use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::api::{UserCreateBody, UserCreateResponse};

impl SyncClient {
    pub fn user_create(
        &self,
        is_enabled: bool,
        name: &str,
        email: &str,
        password: Option<&str>,
    ) -> Result<UserCreateResponse, Error> {
        let body = UserCreateBody {
            is_enabled,
            name: name.to_owned(),
            email: email.to_owned(),
            password: password.map(String::from),
        };

        self.post_json("/v1/user", &body)
            .send()
            .map_err(|_err| Error::Unwrap)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| {
                res.json::<UserCreateResponse>()
                    .map_err(|_err| Error::Unwrap)
            })
    }
}
