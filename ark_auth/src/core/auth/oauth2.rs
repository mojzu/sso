use crate::core::{Error, Service, UserToken};
use crate::driver;

/// OAuth2 user login.
pub fn login(
    driver: &driver::Driver,
    service_id: i64,
    email: &str,
) -> Result<(Service, UserToken), Error> {
    unimplemented!();
}

// TODO(refactor): Refactor this.
// pub fn login(
//     data: &web::Data<ApiData>,
//     email: &str,
//     service_id: i64,
// ) -> Result<(TokenData, AuthService), ApiError> {
//     let token = data
//         .db
//         .login(email, service_id)
//         .map_err(ApiError::Db)?;
//     let service = data
//         .db
//         .service_read_by_id(service_id, service_id)
//         .map_err(ApiError::Db)?;
//     Ok((token, service))
// }
