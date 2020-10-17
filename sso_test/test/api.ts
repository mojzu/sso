import { browser } from "protractor";
import "jasmine";
import {
    api,
    userCreate,
    CLIENT_ID,
    PASSWORD1,
    mailAddress,
    CLIENT_URI,
    SSO_URI,
} from "./util";

describe("sso-api", function () {
    beforeEach(async function () {
        browser.waitForAngularEnabled(false);
    });

    it("should ping server", async function () {
        let response = await api.pingGet();
        expect(response.body).toEqual("ok");
    });

    it("should get well known openid configuration", async function () {
        let response = await api.wellKnownOpenidConfigurationGet();
        expect(response.body).toBeDefined();
        let body = response.body;

        expect(body.issuer).toEqual(`${SSO_URI}/`);
        expect(body.authorizationEndpoint).toEqual(
            `${SSO_URI}/v2/oauth2/authorize`
        );
        expect(body.tokenEndpoint).toEqual(`${SSO_URI}/v2/oauth2/token`);
        expect(body.tokenEndpointAuthMethodsSupported).toEqual([
            "client_secret_basic",
        ]);
    });

    it("should create and verify csrf token", async function () {
        let csrf = await api.v2CsrfCreatePost();
        expect(csrf.body).toBeDefined();
        let token = csrf.body.token;

        let response = await api.v2CsrfVerifyPost({ token });
        expect(response.body).toEqual(null);
    });

    it("should fail to verify invalid csrf token", async function () {
        try {
            await api.v2CsrfVerifyPost({ token: "invalidtoken" });
            fail();
        } catch (e) {
            expect(e.body).toBeDefined();
            expect(e.body.error).toEqual("BadRequest");
            expect(e.body.message).toEqual("csrf token not found or expired");
        }
    });

    it("should read client", async function () {
        let client = (await api.v2ClientReadPost()).body;
        expect(client).toBeDefined();

        expect(client.id).toEqual("b4f765eb-49d9-4d9f-bd4b-8c4b88850f84");
        expect(client.name).toEqual("App");
        expect(client.uri).toEqual(CLIENT_URI);
        expect(client.redirectUri).toEqual(`${CLIENT_URI}oauth2`);
        expect(client.userScope).toEqual("admin api");
        expect(client.registerEnable).toEqual(true);
        expect(client.registerScope).toEqual("");
    });

    it("should read client access", async function () {
        let access = (await api.v2ClientAccessReadPost({})).body;
        expect(access).toBeDefined();
        expect(access.data).toBeDefined();
        expect(access.data.length).toBeGreaterThan(0);
    });

    it("should update client access", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        let access = (
            await api.v2ClientAccessUpdatePost({
                enable: false,
                scope: " api  ",
                userId: user.id,
            })
        ).body;
        expect(access).toBeDefined();
        expect(access.clientId).toEqual(CLIENT_ID);
        expect(access.userId).toEqual(user.id);
        expect(access.enable).toEqual(false);
        expect(access.scope).toEqual("api");
    });

    it("should fail to update client access for unknown scope", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        try {
            await api.v2ClientAccessUpdatePost({
                enable: false,
                scope: "unknownscope",
                userId: user.id,
            });
            fail();
        } catch (e) {
            expect(e.statusCode).toEqual(400);
            expect(e.response.body.error).toEqual("BadRequest");
            expect(e.response.body.message).toEqual("scope invalid");
        }
    });

    it("should delete client access", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        await api.v2ClientAccessDeletePost({
            userId: user.id,
        });

        try {
            await api.v2UserAccessReadPost({
                userId: user.id,
            });
            fail();
        } catch (e) {
            expect(e.statusCode).toEqual(400);
            expect(e.response.body.error).toEqual("BadRequest");
            expect(e.response.body.message).toEqual("access not found");
        }
    });

    it("should create and read audit log", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });
        let audit = (
            await api.v2AuditCreatePost({
                userId: user.id,
                auditType: "test_audit_type",
                subject: "42",
                data: { key: "foo" },
            })
        ).body;
        expect(audit).toBeDefined();
        expect(audit.clientId).toEqual(CLIENT_ID);
        expect(audit.userId).toEqual(user.id);
        expect(audit.auditType).toEqual("test_audit_type");
        expect(audit.subject).toEqual("42");
        expect(audit.data).toEqual({ key: "foo" });

        let auditRead = (
            await api.v2AuditReadPost({
                seek: { limit: 10 },
                id: [audit.id],
                auditType: ["test_audit_type"],
                subject: ["42"],
                userId: [user.id],
            })
        ).body;
        expect(auditRead).toBeDefined();
        expect(auditRead.data.length).toEqual(1);
        expect(auditRead.data[0].id).toEqual(audit.id);
    });

    it("should read audit logs", async function () {
        let read = (
            await api.v2AuditReadPost({
                seek: { limit: 10 },
            })
        ).body;
        expect(read).toBeDefined();
        expect(read.data).toBeDefined();
        expect(read.data.length).toBeGreaterThan(0);
    });

    it("should create user", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        expect(user.id).toBeDefined();
        expect(user.email).toBeDefined();
        expect(user.name).toEqual("test");
        expect(user.enable).toEqual(true);
        expect(user.locale).toEqual("");
        expect(user.timezone).toEqual("");
        expect(user.password).toBeDefined();
        expect(user.password.allowReset).toEqual(true);
        expect(user.password.requireUpdate).toEqual(false);
        expect(user.access).toBeDefined();
    });

    it("should fail to create user with invalid email", async function () {
        try {
            await api.v2UserCreatePost({
                email: "notavalidemailaddress",
                enable: true,
                locale: "",
                name: "test",
                scope: "",
                timezone: "",
                password: null,
            });
            fail();
        } catch (e) {
            expect(e.statusCode).toEqual(400);
            expect(e.response.body.error).toEqual("BadRequest");
            expect(e.response.body.message).toEqual("validation failed");
        }
    });

    it("should fail to create user with too short or long password", async function () {
        try {
            await userCreate({
                password: "guest",
                allowReset: true,
                requireUpdate: false,
            });
        } catch (e) {
            expect(e.statusCode).toEqual(400);
            expect(e.response.body.error).toEqual("BadRequest");
            expect(e.response.body.message).toEqual("validation failed");
        }
        try {
            await userCreate({
                password:
                    "guestguestguestguestguestguestguestguestguestguestguestguestguest",
                allowReset: true,
                requireUpdate: false,
            });
        } catch (e) {
            expect(e.statusCode).toEqual(400);
            expect(e.response.body.error).toEqual("BadRequest");
            expect(e.response.body.message).toEqual("validation failed");
        }
    });

    it("should fail to create user with invalid locale", async function () {
        try {
            await api.v2UserCreatePost({
                email: mailAddress(),
                enable: true,
                locale: "notalocale",
                name: "test",
                scope: "",
                timezone: "",
                password: null,
            });
            fail();
        } catch (e) {
            expect(e.statusCode).toEqual(400);
            expect(e.response.body.error).toEqual("BadRequest");
            expect(e.response.body.message).toEqual("validation failed");
        }
    });

    it("should create user with valid locale", async function () {
        const user = await api.v2UserCreatePost({
            email: mailAddress(),
            enable: true,
            locale: "en-GB",
            name: "test",
            scope: "",
            timezone: "",
            password: null,
        });
        expect(user.body).toBeDefined();
        expect(user.body.locale).toEqual("en-GB");
        expect(user.body.timezone).toEqual("");
    });

    it("should fail to create user with invalid timezone", async function () {
        try {
            await api.v2UserCreatePost({
                email: mailAddress(),
                enable: true,
                locale: "",
                name: "test",
                scope: "",
                timezone: "notatimezone",
                password: null,
            });
            fail();
        } catch (e) {
            expect(e.statusCode).toEqual(400);
            expect(e.response.body.error).toEqual("BadRequest");
            expect(e.response.body.message).toEqual("validation failed");
        }
    });

    it("should create user with valid timezone", async function () {
        const user = await api.v2UserCreatePost({
            email: mailAddress(),
            enable: true,
            locale: "",
            name: "test",
            scope: "",
            timezone: "Etc/UTC",
            password: null,
        });
        expect(user.body).toBeDefined();
        expect(user.body.timezone).toEqual("Etc/UTC");
        expect(user.body.locale).toEqual("");
    });

    it("should read users", async function () {
        let read = (await api.v2UserReadPost({})).body;
        expect(read).toBeDefined();
        expect(read.data).toBeDefined();
        expect(read.data.length).toBeGreaterThan(0);
    });

    it("should update user", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        let updateUser = (
            await api.v2UserUpdatePost({
                id: user.id,
                name: "Update Name",
            })
        ).body;
        expect(updateUser).toBeDefined();
        expect(updateUser.id).toEqual(user.id);
        expect(updateUser.name).toEqual("Update Name");
        expect(updateUser.password).toBeDefined();
        expect(updateUser.password.allowReset).toEqual(true);

        let updateUser2 = (
            await api.v2UserUpdatePost({
                id: user.id,
                password: {
                    allowReset: false,
                    requireUpdate: true,
                },
            })
        ).body;
        expect(updateUser2).toBeDefined();
        expect(updateUser2.id).toEqual(user.id);
        expect(updateUser2.password).toBeDefined();
        expect(updateUser2.password.allowReset).toEqual(false);
        expect(updateUser2.password.requireUpdate).toEqual(true);
    });

    it("should delete user", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        await api.v2UserDeletePost({
            id: user.id,
        });
        let users = (
            await api.v2UserReadPost({
                id: [user.id],
            })
        ).body;
        expect(users).toBeDefined();
        expect(users.data).toBeDefined();
        expect(users.data.length).toEqual(0);
    });

    it("should read user access", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        let access = (
            await api.v2UserAccessReadPost({
                userId: user.id,
            })
        ).body;
        expect(access).toBeDefined();
        expect(access.enable).toEqual(true);
        expect(access.scope).toEqual("");
    });

    it("should fail to update user access for unknown scope", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        try {
            await api.v2UserAccessUpdatePost({
                enable: false,
                scope: "unknownscope",
                userId: user.id,
            });
            fail();
        } catch (e) {
            expect(e.statusCode).toEqual(400);
            expect(e.response.body.error).toEqual("BadRequest");
            expect(e.response.body.message).toEqual("scope invalid");
        }
    });

    it("should delete user access", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        await api.v2UserAccessDeletePost({
            userId: user.id,
        });
        try {
            await api.v2UserAccessReadPost({
                userId: user.id,
            });
            fail();
        } catch (e) {
            expect(e.statusCode).toEqual(400);
            expect(e.response.body.error).toEqual("BadRequest");
            expect(e.response.body.message).toEqual("access not found");
        }
    });

    it("should create and verify api key", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        let apiKey = (
            await api.v2UserApiKeyCreatePost({
                userId: user.id,
                name: "test",
                enable: true,
                scope: "",
            })
        ).body;
        expect(apiKey).toBeDefined();
        expect(apiKey.clientId).toEqual(CLIENT_ID);
        expect(apiKey.userId).toEqual(user.id);
        expect(apiKey.name).toEqual("test");
        expect(apiKey.enable).toEqual(true);
        expect(apiKey.scope).toEqual("");
        expect(apiKey.value).toBeDefined();

        let verify = (
            await api.v2UserApiKeyVerifyPost({
                key: apiKey.value,
            })
        ).body;
        expect(verify).toBeDefined();
        expect(verify.id).toEqual(apiKey.id);
        expect(verify.clientId).toEqual(CLIENT_ID);
        expect(verify.userId).toEqual(user.id);
        expect(verify.value).toBeNull();
    });

    it("should fail to create api key for unknown scope", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        try {
            await api.v2UserApiKeyCreatePost({
                userId: user.id,
                name: "test",
                enable: true,
                scope: "unknownscope",
            });
            fail();
        } catch (e) {
            expect(e.statusCode).toEqual(400);
            expect(e.response.body.error).toEqual("BadRequest");
            expect(e.response.body.message).toEqual("scope invalid");
        }
    });

    it("should fail to verify invalid api key", async function () {
        try {
            await api.v2UserApiKeyVerifyPost({
                key:
                    "b4f765eb-49d9-4d9f-bd4b-8c4b88850f84.J0q+oMl+jCXdglu2RCEWJd6vBpxzg+dTIPJzyNkUMKE=",
            });
            fail();
        } catch (e) {
            expect(e.response.body).toBeDefined();
            expect(e.response.body.error).toEqual("BadRequest");
            expect(e.response.body.message).toEqual("api key verify failed");
        }
    });

    it("should read api key", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });
        let apiKey = (
            await api.v2UserApiKeyCreatePost({
                userId: user.id,
                name: "test",
                enable: true,
                scope: "",
            })
        ).body;

        let read = (
            await api.v2UserApiKeyReadPost({
                id: [apiKey.id],
            })
        ).body;
        expect(read).toBeDefined();
        expect(read.data).toBeDefined();
        expect(read.data.length).toEqual(1);
        expect(read.data[0].id).toEqual(apiKey.id);
    });

    it("should update api key", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });
        let apiKey = (
            await api.v2UserApiKeyCreatePost({
                userId: user.id,
                name: "test",
                enable: true,
                scope: "",
            })
        ).body;

        let update = (
            await api.v2UserApiKeyUpdatePost({
                id: apiKey.id,
                name: "Update Name",
                enable: false,
            })
        ).body;
        expect(update).toBeDefined();
        expect(update.id).toEqual(apiKey.id);
        expect(update.name).toEqual("Update Name");
        expect(update.enable).toEqual(false);
    });

    it("should fail to update api key for invalid name", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });
        let apiKey = (
            await api.v2UserApiKeyCreatePost({
                userId: user.id,
                name: "test",
                enable: true,
                scope: "",
            })
        ).body;

        try {
            await api.v2UserApiKeyUpdatePost({
                id: apiKey.id,
                name: "",
                enable: false,
            });
            fail();
        } catch (e) {
            expect(e.statusCode).toEqual(400);
            expect(e.response.body.error).toEqual("BadRequest");
            expect(e.response.body.message).toEqual("validation failed");
        }
    });

    it("should delete api key", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });
        let apiKey = (
            await api.v2UserApiKeyCreatePost({
                userId: user.id,
                name: "test",
                enable: true,
                scope: "",
            })
        ).body;
        await api.v2UserApiKeyDeletePost({
            id: apiKey.id,
        });

        try {
            await api.v2UserApiKeyVerifyPost({
                key: apiKey.value,
            });
            fail();
        } catch (e) {
            expect(e.response.body).toBeDefined();
            expect(e.response.body.error).toEqual("BadRequest");
            expect(e.response.body.message).toEqual("api key verify failed");
        }
    });
});
