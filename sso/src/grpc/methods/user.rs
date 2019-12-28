use crate::{
    api::{self, ApiError, ApiResult, ValidateRequest},
    grpc::{pb, util::*},
    *,
};
use std::convert::TryInto;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

pub async fn list(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::UserListRequest>,
) -> Result<Response<pb::UserListReply>, Status> {
    unimplemented!();
}

pub async fn create(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::UserCreateRequest>,
) -> Result<Response<pb::UserReadReply>, Status> {
    unimplemented!();
}

pub async fn read(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::UserReadRequest>,
) -> Result<Response<pb::UserReadReply>, Status> {
    unimplemented!();
}

pub async fn update(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::UserUpdateRequest>,
) -> Result<Response<pb::UserReadReply>, Status> {
    unimplemented!();
}

pub async fn delete(
    driver: Arc<Box<dyn Driver>>,
    request: Request<pb::UserReadRequest>,
) -> Result<Response<()>, Status> {
    unimplemented!();
}
