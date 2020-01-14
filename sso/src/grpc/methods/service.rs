use crate::{
    grpc::{pb, util::*, validate, Server},
    *,
};
use tonic::Response;
use uuid::Uuid;
use validator::{Validate, ValidationErrors};

impl Validate for pb::ServiceListRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid_opt(e, "gt", self.gt.as_ref().map(|x| &**x));
            validate::uuid_opt(e, "lt", self.lt.as_ref().map(|x| &**x));
            validate::limit_opt(e, "limit", self.limit);
            validate::uuid_vec(e, "id", &self.id);
        })
    }
}

pub async fn list(
    server: &Server,
    request: MethodRequest<ServiceList>,
) -> MethodResponse<pb::ServiceListReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, MethodError, _>(move || {
        let data = audit_result_err(
            driver.as_ref().as_ref(),
            audit_meta,
            AuditType::ServiceList,
            |driver, audit| {
                pattern::key_root_authenticate2(driver, audit, auth.as_ref())
                    .map_err(MethodError::Unauthorised)?;

                driver.service_list(&req).map_err(MethodError::BadRequest)
            },
        )?;
        let reply = pb::ServiceListReply {
            meta: Some(req.into()),
            data: data
                .into_iter()
                .map::<pb::Service, _>(|x| x.into())
                .collect(),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

impl Validate for pb::ServiceCreateRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::name(e, "name", &self.name);
            validate::url(e, "url", &self.url);
            validate::text_opt(
                e,
                "user_email_text",
                self.user_email_text.as_ref().map(|x| &**x),
            );
            validate::url_opt(
                e,
                "provider_local_url",
                self.provider_local_url.as_ref().map(|x| &**x),
            );
            validate::url_opt(
                e,
                "provider_github_oauth2_url",
                self.provider_github_oauth2_url.as_ref().map(|x| &**x),
            );
            validate::url_opt(
                e,
                "provider_microsoft_oauth2_url",
                self.provider_microsoft_oauth2_url.as_ref().map(|x| &**x),
            );
        })
    }
}

pub async fn create(
    server: &Server,
    request: MethodRequest<ServiceCreate>,
) -> MethodResponse<pb::ServiceReadReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, MethodError, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::ServiceCreate);
        let res: Result<Service, MethodError> = {
            pattern::key_root_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(MethodError::Unauthorised)?;

            driver.service_create(&req).map_err(MethodError::BadRequest)
        };
        let data = audit_result_subject(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::ServiceReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

impl Validate for pb::ServiceReadRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid(e, "id", &self.id);
        })
    }
}

pub async fn read(
    server: &Server,
    request: MethodRequest<ServiceRead>,
) -> MethodResponse<pb::ServiceReadReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, MethodError, _>(move || {
        let data = audit_result_err(
            driver.as_ref().as_ref(),
            audit_meta,
            AuditType::ServiceRead,
            |driver, audit| {
                let service = pattern::key_authenticate2(driver, audit, auth.as_ref())
                    .map_err(MethodError::Unauthorised)?;

                read_inner(driver, &req, service.map(|x| x.id))
            },
        )?;
        let reply = pb::ServiceReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

impl Validate for pb::ServiceUpdateRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        validate::wrap(|e| {
            validate::uuid(e, "id", &self.id);
            validate::name_opt(e, "name", self.name.as_ref().map(|x| &**x));
            validate::url_opt(e, "url", self.url.as_ref().map(|x| &**x));
            validate::text_opt(
                e,
                "user_email_text",
                self.user_email_text.as_ref().map(|x| &**x),
            );
            validate::url_opt(
                e,
                "provider_local_url",
                self.provider_local_url.as_ref().map(|x| &**x),
            );
            validate::url_opt(
                e,
                "provider_github_oauth2_url",
                self.provider_github_oauth2_url.as_ref().map(|x| &**x),
            );
            validate::url_opt(
                e,
                "provider_microsoft_oauth2_url",
                self.provider_microsoft_oauth2_url.as_ref().map(|x| &**x),
            );
        })
    }
}

pub async fn update(
    server: &Server,
    request: MethodRequest<ServiceUpdate>,
) -> MethodResponse<pb::ServiceReadReply> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, MethodError, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::ServiceUpdate);
        let res: Result<(Service, Service), MethodError> = {
            let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(MethodError::Unauthorised)?;

            let read = ServiceRead::new(req.id);
            let previous_service =
                read_inner(driver.as_ref().as_ref(), &read, service.map(|x| x.id))?;
            let service = driver
                .service_update(&req)
                .map_err(MethodError::BadRequest)?;
            Ok((previous_service, service))
        };
        let data = audit_result_diff(driver.as_ref().as_ref(), &audit, res)?;
        let reply = pb::ServiceReadReply {
            data: Some(data.into()),
        };
        Ok(reply)
    })
    .await?;
    Ok(Response::new(reply))
}

pub async fn delete(server: &Server, request: MethodRequest<ServiceRead>) -> MethodResponse<()> {
    let (audit_meta, auth, req) = request.into_inner();

    let driver = server.driver();
    let reply = blocking::<_, MethodError, _>(move || {
        let mut audit = AuditBuilder::new(audit_meta, AuditType::ServiceDelete);
        let res: Result<Service, MethodError> = {
            let service = pattern::key_authenticate(driver.as_ref().as_ref(), &mut audit, auth)
                .map_err(MethodError::Unauthorised)?;

            let service = read_inner(driver.as_ref().as_ref(), &req, service.map(|x| x.id))?;
            driver
                .service_delete(&service.id)
                .map_err(MethodError::BadRequest)
                .map(|_| service)
        };
        audit_result_subject(driver.as_ref().as_ref(), &audit, res)?;
        Ok(())
    })
    .await?;
    Ok(Response::new(reply))
}

fn read_inner(
    driver: &dyn Driver,
    read: &ServiceRead,
    service_id: Option<Uuid>,
) -> MethodResult<Service> {
    driver
        .service_read(read, service_id)
        .map_err(MethodError::BadRequest)?
        .ok_or_else(|| DriverError::ServiceNotFound)
        .map_err(MethodError::NotFound)
}
