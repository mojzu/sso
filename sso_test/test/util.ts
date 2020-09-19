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
import * as fs from "fs";
import * as path from "path";
import * as urijs from "urijs";
import {
    DefaultApi,
    RequestUserCreatePassword,
    ResponseUser,
} from "../client/api";

export const CLIENT_DOMAIN = "http://localhost:7042";
export const AUTHORIZE_URI = `${CLIENT_DOMAIN}/v2/oauth2/authorize`;
export const TOKEN_URI = `${CLIENT_DOMAIN}/v2/oauth2/token`;
export const CLIENT_ID = "b4f765eb-49d9-4d9f-bd4b-8c4b88850f84";
export const CLIENT_SECRET = "QypqqfAUyzv4hu8lQWrRKjgsxr22UzaMKvvkbwBzkMw=";
export const REDIRECT_URI = "http://localhost:8080/oauth2";
export const AUTH_URI = `${CLIENT_DOMAIN}/v2/auth`;

export const PASSWORD1 = "guestguest";
export const PASSWORD2 = "guestfoobar";

export function mailAddress(): string {
    let name = "";
    let char = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let charLen = char.length;
    for (let i = 0; i < 12; i++) {
        name += char.charAt(Math.floor(Math.random() * charLen));
    }
    return `${name}@test.localhost`;
}

export type Mail = { to: string; subject: string; text: string };

export function mailRead(email: string, subject: string): Mail {
    let filePath = path.normalize("/opt/mailto/mailto.log");
    let content = fs.readFileSync(filePath, { encoding: "utf8" });
    let contentSplit = content.split(/\r?\n/).filter((x) => x !== "");

    for (const line of contentSplit.reverse()) {
        const obj = JSON.parse(line);
        if (obj.to === email) {
            expect(obj.to).toEqual(email);
            expect(obj.subject).toEqual(subject);
            return obj;
        }
    }
    return null;
}

export function mailUrls(mail: Mail): string[] {
    let urls = [];
    urijs.withinString(mail.text, (x) => {
        urls.push(x);
    });
    return urls;
}

export async function browser_sleep(i: number = 1) {
    await browser.sleep(i * 1000);
}

export async function userCreate(
    password?: RequestUserCreatePassword,
    enable: boolean = true
): Promise<ResponseUser> {
    let response = await api.v2UserCreatePost({
        email: mailAddress(),
        enable,
        locale: "",
        name: "test",
        scope: "",
        timezone: "",
        password,
    });
    return response.body;
}

export async function form_password_login_submit(
    email: string,
    password: string
) {
    let emailInput = await browser.findElement(By.id("password-login-email"));
    await emailInput.sendKeys(email);

    let passwordInput = await browser.findElement(
        By.id("password-login-password")
    );
    await passwordInput.sendKeys(password);

    let submit = await browser.findElement(By.id("password-login-submit"));
    await submit.click();
    await browser_sleep();
}

export async function form_delete_submit(password: string) {
    let passwordInput = await browser.findElement(By.id("password"));
    await passwordInput.sendKeys(password);

    let submit = await browser.findElement(By.id("submit"));
    await submit.click();
    await browser_sleep();
}

export async function form_delete_accept_submit(
    userEmail: string,
    password: string
) {
    let mail = mailRead(userEmail, "Delete Request");
    expect(mail).toBeDefined();

    let urls = mailUrls(mail);
    let accept_url = urls[0];

    browser.get(accept_url);
    expect(await browser.getCurrentUrl()).toContain(
        `${CLIENT_DOMAIN}/v2/auth/delete`
    );

    let passwordEl = await browser.findElement(By.id("password"));
    await passwordEl.sendKeys(password);

    let submit = await browser.findElement(By.id("submit"));
    await submit.click();
    await browser_sleep();
}

export async function form_password_reset_submit(
    userEmail: string,
    passwordNew: string,
    passwordConfirm: string
) {
    let email = await browser.findElement(By.id("password-reset-email"));
    await email.sendKeys(userEmail);

    let submit = await browser.findElement(By.id("password-reset-submit"));
    await submit.click();
    await browser_sleep();

    let mail = mailRead(userEmail, "Password Reset Request");
    expect(mail).toBeDefined();

    let urls = mailUrls(mail);
    let accept_url = urls[0];

    browser.get(accept_url);
    expect(await browser.getCurrentUrl()).toContain(
        `${CLIENT_DOMAIN}/v2/auth/password-reset`
    );

    let passwordNewEl = await browser.findElement(By.id("password-new"));
    await passwordNewEl.sendKeys(passwordNew);

    let passwordConfirmEl = await browser.findElement(
        By.id("password-confirm")
    );
    await passwordConfirmEl.sendKeys(passwordConfirm);

    submit = await browser.findElement(By.id("submit"));
    await submit.click();
    await browser_sleep();
}

export async function form_password_update_submit(
    passwordCurrent: string,
    passwordNew: string,
    passwordConfirm: string
) {
    let passwordEl = await browser.findElement(By.id("password"));
    await passwordEl.sendKeys(passwordCurrent);

    let passwordNewEl = await browser.findElement(By.id("password-new"));
    await passwordNewEl.sendKeys(passwordNew);

    let passwordConfirmEl = await browser.findElement(
        By.id("password-confirm")
    );
    await passwordConfirmEl.sendKeys(passwordConfirm);

    let submit = await browser.findElement(By.id("submit"));
    await submit.click();
    await browser_sleep();
}

export async function form_email_update_submit(
    passwordCurrent: string,
    emailNew: string,
    emailConfirm: string
) {
    let passwordEl = await browser.findElement(By.id("password"));
    await passwordEl.sendKeys(passwordCurrent);

    let emailNewEl = await browser.findElement(By.id("email-new"));
    await emailNewEl.sendKeys(emailNew);

    let emailConfirmEl = await browser.findElement(By.id("email-confirm"));
    await emailConfirmEl.sendKeys(emailConfirm);

    let submit = await browser.findElement(By.id("submit"));
    await submit.click();
    await browser_sleep();
}

export async function form_register_submit(emailAddress: string) {
    let email = await browser.findElement(By.id("register-email"));
    await email.sendKeys(emailAddress);

    let submit = await browser.findElement(By.id("register-submit"));
    await submit.click();
    await browser_sleep();
}

export async function form_register_accept_password(
    emailAddress: string,
    password: string,
    passwordConfirm: string
) {
    let mail = mailRead(emailAddress, "Register Request");
    expect(mail).toBeDefined();

    let urls = mailUrls(mail);
    let accept_url = urls[0];

    browser.get(accept_url);
    expect(await browser.getCurrentUrl()).toContain(`${AUTH_URI}/register`);

    let name = await browser.findElement(By.id("password-name"));
    await name.sendKeys("Foo");

    let passwordEl = await browser.findElement(By.id("password-password"));
    await passwordEl.sendKeys(password);

    let passwordConfirmEl = await browser.findElement(
        By.id("password-password-confirm")
    );
    await passwordConfirmEl.sendKeys(passwordConfirm);

    let submit = await browser.findElement(By.id("password-submit"));
    await submit.click();
    await browser_sleep();
}

export async function form_oauth2_sso_login_submit(
    emailAddress: string,
    password: string
) {
    let submit = await browser.findElement(By.id("oauth2-sso-submit"));
    await submit.click();
    await browser_sleep();

    await form_password_login_submit(emailAddress, password);
}

export async function form_register_accept_oauth2_sso_submit(
    emailAddress: string,
    password: string
) {
    let mail = mailRead(emailAddress, "Register Request");
    expect(mail).toBeDefined();

    let urls = mailUrls(mail);
    let accept_url = urls[0];

    browser.get(accept_url);
    expect(await browser.getCurrentUrl()).toContain(`${AUTH_URI}/register`);

    let submit = await browser.findElement(By.id("oauth2-sso-submit"));
    await submit.click();
    await browser_sleep();

    await form_password_login_submit(emailAddress, password);
}

export async function error_check_code_description(
    code: string,
    description: String
) {
    let codeEl = await browser.findElement(By.id("error-code"));
    let codeCompare = await codeEl.getText();

    let descriptionEl = await browser.findElement(By.id("error-description"));
    let descriptionCompare = await descriptionEl.getText();

    expect(code).toEqual(codeCompare);
    expect(description).toEqual(descriptionCompare);
}

export async function browser_get_authorize() {
    browser.get("http://localhost:8080/");
    expect(await browser.getCurrentUrl()).toContain(AUTHORIZE_URI);
}

export async function browser_get_password_update() {
    let uri = encodeURI(
        `${AUTH_URI}/password-update?client_id=${CLIENT_ID}&redirect_uri=${REDIRECT_URI}`
    );
    browser.get(uri);
    expect(await browser.getCurrentUrl()).toContain(
        `${AUTH_URI}/password-update`
    );
}

export async function browser_get_delete() {
    let uri = encodeURI(
        `${AUTH_URI}/delete?client_id=${CLIENT_ID}&redirect_uri=${REDIRECT_URI}`
    );
    browser.get(uri);
    expect(await browser.getCurrentUrl()).toContain(`${AUTH_URI}/delete`);
}

export async function browser_get_email_update() {
    let uri = encodeURI(
        `${AUTH_URI}/email-update?client_id=${CLIENT_ID}&redirect_uri=${REDIRECT_URI}`
    );
    browser.get(uri);
    expect(await browser.getCurrentUrl()).toContain(`${AUTH_URI}/email-update`);
}

export type Token = { access: string; refresh: string };

export async function browser_check_authorized(): Promise<Token> {
    expect(await browser.getCurrentUrl()).toEqual("http://localhost:8080/");

    let cookie = await browser.manage().getCookie("sso.id");
    expect(cookie).toBeDefined();
    expect(cookie.domain).toEqual("localhost");
    expect(cookie.value).toBeDefined();

    let accessEl = await browser.findElement(By.id("access-token"));
    let access = await accessEl.getText();

    let refreshEl = await browser.findElement(By.id("refresh-token"));
    let refresh = await refreshEl.getText();

    return { access, refresh };
}

export const api = new DefaultApi(CLIENT_ID, CLIENT_SECRET, CLIENT_DOMAIN);

export const api2 = new DefaultApi(
    "f5683aca-4b25-43e4-b6fe-3fb1002ec5fd",
    "0skM1U/uGZScXraYL9hjQ6bAicGvHiFHM1g9dHyJDTs=",
    "http://localhost:7044"
);
