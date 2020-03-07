use crate::prelude::*;

pub async fn local_login(
    server: &GrpcServiceServer,
    request: GrpcMethodRequest<pb::AuthLoginRequest>,
) -> GrpcMethodResult<pb::AuthLoginReply> {
    let (audit_meta, _auth, req) = request.into_inner();
    let res = server
        .client(&audit_meta)
        .auth_local_login(req)
        .await?
        .into_inner();
    Ok(res.into())
}

pub async fn local_register(
    server: &GrpcServiceServer,
    request: GrpcMethodRequest<pb::AuthRegisterRequest>,
) -> GrpcMethodResult<()> {
    let (audit_meta, _auth, req) = request.into_inner();
    let res = server
        .client(&audit_meta)
        .auth_local_register(req)
        .await?
        .into_inner();
    Ok(res.into())
}

pub async fn local_register_confirm(
    server: &GrpcServiceServer,
    request: GrpcMethodRequest<pb::AuthRegisterConfirmRequest>,
) -> GrpcMethodResult<pb::AuthPasswordMetaReply> {
    let (audit_meta, _auth, req) = request.into_inner();
    let res = server
        .client(&audit_meta)
        .auth_local_register_confirm(req)
        .await?
        .into_inner();
    Ok(res.into())
}

pub async fn local_register_revoke(
    server: &GrpcServiceServer,
    request: GrpcMethodRequest<pb::AuthTokenRequest>,
) -> GrpcMethodResult<pb::AuthAuditReply> {
    let (audit_meta, _auth, req) = request.into_inner();
    let res = server
        .client(&audit_meta)
        .auth_local_register_revoke(req)
        .await?
        .into_inner();
    Ok(res.into())
}

pub async fn local_reset_password(
    server: &GrpcServiceServer,
    request: GrpcMethodRequest<pb::AuthResetPasswordRequest>,
) -> GrpcMethodResult<()> {
    let (audit_meta, _auth, req) = request.into_inner();
    let res = server
        .client(&audit_meta)
        .auth_local_reset_password(req)
        .await?
        .into_inner();
    Ok(res.into())
}

pub async fn local_reset_password_confirm(
    server: &GrpcServiceServer,
    request: GrpcMethodRequest<pb::AuthResetPasswordConfirmRequest>,
) -> GrpcMethodResult<pb::AuthPasswordMetaReply> {
    let (audit_meta, _auth, req) = request.into_inner();
    let res = server
        .client(&audit_meta)
        .auth_local_reset_password_confirm(req)
        .await?
        .into_inner();
    Ok(res.into())
}

pub async fn local_reset_password_revoke(
    server: &GrpcServiceServer,
    request: GrpcMethodRequest<pb::AuthTokenRequest>,
) -> GrpcMethodResult<pb::AuthAuditReply> {
    let (audit_meta, _auth, req) = request.into_inner();
    let res = server
        .client(&audit_meta)
        .auth_local_reset_password_revoke(req)
        .await?
        .into_inner();
    Ok(res.into())
}

pub async fn local_update_email(
    server: &GrpcServiceServer,
    request: GrpcMethodRequest<pb::AuthUpdateEmailRequest>,
) -> GrpcMethodResult<()> {
    let (audit_meta, _auth, req) = request.into_inner();
    let res = server
        .client(&audit_meta)
        .auth_local_update_email(req)
        .await?
        .into_inner();
    Ok(res.into())
}

pub async fn local_update_email_revoke(
    server: &GrpcServiceServer,
    request: GrpcMethodRequest<pb::AuthTokenRequest>,
) -> GrpcMethodResult<pb::AuthAuditReply> {
    let (audit_meta, _auth, req) = request.into_inner();
    let res = server
        .client(&audit_meta)
        .auth_local_update_email_revoke(req)
        .await?
        .into_inner();
    Ok(res.into())
}

pub async fn local_update_password(
    server: &GrpcServiceServer,
    request: GrpcMethodRequest<pb::AuthUpdatePasswordRequest>,
) -> GrpcMethodResult<pb::AuthPasswordMetaReply> {
    let (audit_meta, _auth, req) = request.into_inner();
    let res = server
        .client(&audit_meta)
        .auth_local_update_password(req)
        .await?
        .into_inner();
    Ok(res.into())
}

pub async fn local_update_password_revoke(
    server: &GrpcServiceServer,
    request: GrpcMethodRequest<pb::AuthTokenRequest>,
) -> GrpcMethodResult<pb::AuthAuditReply> {
    let (audit_meta, _auth, req) = request.into_inner();
    let res = server
        .client(&audit_meta)
        .auth_local_update_password_revoke(req)
        .await?
        .into_inner();
    Ok(res.into())
}

pub async fn microsoft_oauth2_url(
    server: &GrpcServiceServer,
    request: GrpcMethodRequest<()>,
) -> GrpcMethodResult<pb::AuthOauth2UrlReply> {
    let (audit_meta, _auth, req) = request.into_inner();
    let res = server
        .client(&audit_meta)
        .auth_microsoft_oauth2_url(req)
        .await?
        .into_inner();
    Ok(res.into())
}

pub async fn microsoft_oauth2_callback(
    server: &GrpcServiceServer,
    request: GrpcMethodRequest<pb::AuthOauth2CallbackRequest>,
) -> GrpcMethodResult<pb::AuthTokenReply> {
    let (audit_meta, _auth, req) = request.into_inner();
    let res = server
        .client(&audit_meta)
        .auth_microsoft_oauth2_callback(req)
        .await?
        .into_inner();
    Ok(res.into())
}

pub async fn token_refresh(
    server: &GrpcServiceServer,
    request: GrpcMethodRequest<pb::AuthTokenRequest>,
) -> GrpcMethodResult<pb::AuthTokenReply> {
    let (audit_meta, _auth, req) = request.into_inner();
    let res = server
        .client(&audit_meta)
        .auth_token_refresh(req)
        .await?
        .into_inner();
    Ok(res.into())
}
