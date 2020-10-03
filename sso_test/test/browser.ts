import {
    browser,
    element,
    by,
    By,
    $,
    $$,
    ExpectedConditions,
} from "protractor";
import "jasmine";
import {
    api,
    browser_get_authorize,
    mailAddress,
    form_password_reset_submit,
    userCreate,
    form_password_login_submit,
    error_check_code_description,
    browser_check_authorized,
    form_register_submit,
    form_register_accept_password,
    browser_get_password_update,
    browser_get_email_update,
    form_password_update_submit,
    form_email_update_submit,
    browser_get_delete,
    form_delete_accept_submit,
    form_delete_submit,
    PASSWORD1,
    PASSWORD2,
    AUTH_URI,
    form_register_accept_oauth2_sso_submit,
    form_oauth2_sso_login_submit,
    api2,
    AUTHORIZE_URI,
    CLIENT_URI,
    browser_delete_cookies,
} from "./util";

describe("sso-browser", function () {
    beforeEach(async function () {
        browser.waitForAngularEnabled(false);
        await browser_delete_cookies();
    });

    it("should login successfully", async function () {
        await browser_get_authorize();

        await form_password_login_submit(
            "admin@app.dev",
            "daH1PqPo08fifuZVI2RuaW2jUg7KAQK0TncPNwAqswE="
        );

        await browser_check_authorized();
    });

    it("should fail to login with unknown email", async function () {
        await browser_get_authorize();

        await form_password_login_submit(
            "thisemailisunknown@app.dev",
            PASSWORD1
        );

        await error_check_code_description("access_denied", "email not found");
    });

    it("should fail to login with wrong password", async function () {
        await browser_get_authorize();

        await form_password_login_submit(
            "admin@app.dev",
            "thispasswordiswrong"
        );

        await error_check_code_description(
            "access_denied",
            "password is incorrect"
        );
    });

    it("should fail to login if user not enabled", async function () {
        let user = await userCreate(
            {
                password: PASSWORD1,
                allowReset: true,
                requireUpdate: false,
            },
            false
        );

        await browser_get_authorize();

        await form_password_login_submit(user.email, PASSWORD1);

        await error_check_code_description("access_denied", "user is disabled");
    });

    it("should fail to login if user access not enabled", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });
        await api.v2UserAccessUpdatePost({
            userId: user.id,
            enable: false,
            scope: "",
        });

        await browser_get_authorize();

        await form_password_login_submit(user.email, PASSWORD1);

        await error_check_code_description(
            "access_denied",
            "access is disabled"
        );
    });

    it("should fail to login with too short or long password", async function () {
        await browser_get_authorize();

        await form_password_login_submit("admin@app.dev", "guest");
        // browser behaviour on minlength is to display error message
        expect(await browser.getCurrentUrl()).toContain(AUTHORIZE_URI);

        await browser_get_authorize();

        await form_password_login_submit(
            "admin@app.dev",
            "guestguestguestguestguestguestguestguestguestguestguestguestguest"
        );
        // browser behaviour on exceeding maxlength is to submit with value
        // truncated to fit
        await error_check_code_description(
            "access_denied",
            "password is incorrect"
        );
    });

    it("should require password update to login", async function () {
        let user = await userCreate(
            {
                password: PASSWORD1,
                allowReset: true,
                requireUpdate: true,
            },
            true
        );

        await browser_get_authorize();

        await form_password_login_submit(user.email, PASSWORD1);

        expect(await browser.getCurrentUrl()).toContain(
            `${AUTH_URI}/password-update`
        );

        await form_password_update_submit(PASSWORD1, PASSWORD2, PASSWORD2);

        browser.get(CLIENT_URI);

        await browser_check_authorized();
    });

    it("should reset user password", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        await browser_get_authorize();

        await form_password_reset_submit(user.email, PASSWORD2, PASSWORD2);

        await browser_get_authorize();

        await form_password_login_submit(user.email, PASSWORD2);

        await browser_check_authorized();
    });

    it("should fail to reset user password", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        await browser_get_authorize();

        await form_password_reset_submit(user.email, PASSWORD1, PASSWORD2);

        await error_check_code_description(
            "invalid_request",
            "password_new does not match password_confirm"
        );
    });

    it("should update user email", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        await browser_get_authorize();

        await form_password_login_submit(user.email, PASSWORD1);

        await browser_get_email_update();

        let email = mailAddress();
        await form_email_update_submit(PASSWORD1, email, email);

        await browser_delete_cookies();

        await browser_get_authorize();

        await form_password_login_submit(user.email, PASSWORD1);

        await error_check_code_description("access_denied", "email not found");

        await form_password_login_submit(email, PASSWORD1);

        await browser_check_authorized();
    });

    it("should fail to update user email", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        await browser_get_authorize();

        await form_password_login_submit(user.email, PASSWORD1);

        await browser_get_email_update();

        let email = mailAddress();
        await form_email_update_submit(PASSWORD2, email, email);

        await error_check_code_description(
            "server_error",
            "email update failed"
        );
    });

    it("should update user password", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        await browser_get_authorize();

        await form_password_login_submit(user.email, PASSWORD1);

        await browser_get_password_update();

        await form_password_update_submit(PASSWORD1, PASSWORD2, PASSWORD2);

        await browser_delete_cookies();

        await browser_get_authorize();

        await form_password_login_submit(user.email, PASSWORD1);

        await error_check_code_description(
            "access_denied",
            "password is incorrect"
        );

        await form_password_login_submit(user.email, PASSWORD2);

        await browser_check_authorized();
    });

    it("should fail to update user password", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        await browser_get_authorize();

        await form_password_login_submit(user.email, PASSWORD1);

        await browser_get_password_update();

        await form_password_update_submit(PASSWORD2, PASSWORD2, PASSWORD2);

        await error_check_code_description(
            "server_error",
            "password update failed"
        );
    });

    it("should register using password successfully", async function () {
        await browser_get_authorize();

        let email = mailAddress();
        await form_register_submit(email);

        await form_register_accept_password(email, PASSWORD1, PASSWORD1);

        browser.get(CLIENT_URI);

        await browser_check_authorized();
    });

    it("should fail to register with password mismatch", async function () {
        await browser_get_authorize();

        let email = mailAddress();
        await form_register_submit(email);

        await form_register_accept_password(email, PASSWORD1, PASSWORD2);

        await error_check_code_description(
            "invalid_request",
            "password does not match password_confirm"
        );
    });

    it("should register and login using oauth2 provider successfully", async function () {
        let user = (
            await api2.v2UserCreatePost({
                email: mailAddress(),
                enable: true,
                locale: "",
                name: "test",
                scope: "",
                timezone: "",
                password: {
                    password: PASSWORD1,
                    allowReset: true,
                    requireUpdate: false,
                },
            })
        ).body;

        await browser_get_authorize();

        await form_register_submit(user.email);

        await form_register_accept_oauth2_sso_submit(user.email, PASSWORD1);

        browser.get(CLIENT_URI);

        await browser_check_authorized();

        await browser_delete_cookies();

        await browser_get_authorize();

        await form_oauth2_sso_login_submit(user.email, PASSWORD1);

        await browser_check_authorized();
    });

    it("should delete user", async function () {
        let user = await userCreate({
            password: PASSWORD1,
            allowReset: true,
            requireUpdate: false,
        });

        await browser_get_authorize();

        await form_password_login_submit(user.email, PASSWORD1);

        await browser_get_delete();

        await form_delete_submit(PASSWORD1);

        await form_delete_accept_submit(user.email, PASSWORD1);

        await browser_delete_cookies();

        await browser_get_authorize();

        await form_password_login_submit(user.email, PASSWORD1);

        await error_check_code_description("access_denied", "email not found");
    });

    it("should introspect token", async function () {
        await browser_get_authorize();

        await form_password_login_submit(
            "admin@app.dev",
            "daH1PqPo08fifuZVI2RuaW2jUg7KAQK0TncPNwAqswE="
        );

        let token = await browser_check_authorized();

        let response = await api.v2Oauth2IntrospectPost({
            token: token.access,
        });
        let introspectToken = response.body;

        expect(introspectToken.active).toEqual(true);
        expect(introspectToken.client_id).toEqual(
            "b4f765eb-49d9-4d9f-bd4b-8c4b88850f84"
        );
        expect(introspectToken.scope).toEqual("admin api");
        expect(introspectToken.username).toEqual("Admin");
    });

    it("should refresh tokens", async function () {
        await browser_get_authorize();

        await form_password_login_submit(
            "admin@app.dev",
            "daH1PqPo08fifuZVI2RuaW2jUg7KAQK0TncPNwAqswE="
        );

        let token = await browser_check_authorized();

        let response = await api.v2Oauth2TokenPost({
            grantType: "refresh_token",
            refreshToken: token.refresh,
        });
        let newToken = response.body;

        expect(newToken.access_token).toBeDefined();
        expect(newToken.refresh_token).toBeDefined();
        expect(newToken.access_token).not.toEqual(token.access);
        expect(newToken.refresh_token).not.toEqual(token.refresh);
    });
});
