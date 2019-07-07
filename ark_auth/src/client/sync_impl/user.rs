use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::api::{UserCreateBody, UserCreateResponse, UserListQuery, UserListResponse};

impl SyncClient {
    pub fn user_list(
        &self,
        gt: Option<&str>,
        lt: Option<&str>,
        limit: Option<i64>,
        email_eq: Option<&str>,
    ) -> Result<UserListResponse, Error> {
        let query = UserListQuery {
            gt: gt.map(|x| x.to_owned()),
            lt: lt.map(|x| x.to_owned()),
            limit: limit.map(|x| format!("{}", x)),
            email_eq: email_eq.map(|x| x.to_owned()),
        };

        self.get_query("/v1/user", query)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<UserListResponse>().map_err(Into::into))
    }

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
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<UserCreateResponse>().map_err(Into::into))
    }
}
