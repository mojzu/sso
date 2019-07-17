use crate::client::sync_impl::SyncClient;
use crate::client::Error;
use crate::server::api::{
    route, UserCreateBody, UserCreateResponse, UserListQuery, UserListResponse, UserReadResponse,
    UserUpdateBody,
};

impl SyncClient {
    pub fn user_list(&self, query: UserListQuery) -> Result<UserListResponse, Error> {
        self.get_query(route::USER, query)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<UserListResponse>().map_err(Into::into))
    }

    pub fn user_create<T: Into<String>>(
        &self,
        is_enabled: bool,
        name: T,
        email: T,
        password: Option<String>,
    ) -> Result<UserCreateResponse, Error> {
        let body = UserCreateBody {
            is_enabled,
            name: name.into(),
            email: email.into(),
            password,
        };
        self.post_json(route::USER, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<UserCreateResponse>().map_err(Into::into))
    }

    pub fn user_read(&self, id: &str) -> Result<UserReadResponse, Error> {
        let path = route::user_id(id);
        self.get(&path)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<UserReadResponse>().map_err(Into::into))
    }

    pub fn user_update(
        &self,
        id: &str,
        is_enabled: Option<bool>,
        name: Option<String>,
    ) -> Result<UserReadResponse, Error> {
        let path = route::user_id(id);
        let body = UserUpdateBody { is_enabled, name };
        self.patch_json(&path, &body)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .and_then(|mut res| res.json::<UserReadResponse>().map_err(Into::into))
    }

    pub fn user_delete(&self, id: &str) -> Result<(), Error> {
        let path = route::user_id(id);
        self.delete(&path)
            .send()
            .map_err(Into::into)
            .and_then(SyncClient::match_status_code)
            .map(|_res| ())
    }
}
