import * as fs from "fs";
import * as path from "path";
import * as proc from "child_process";

const minifyHtml = require("html-minifier").minify;

const indexHtmlIn = path.normalize("./template/index.html");
const indexHtml = fs.readFileSync(indexHtmlIn, { encoding: "utf8" });

const errorHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/error.hbs"),
    { encoding: "utf8" }
);
const authHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/auth.hbs"),
    { encoding: "utf8" }
);
const passwordResetRequestHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/password_reset/request.hbs"),
    { encoding: "utf8" }
);
const passwordResetAcceptHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/password_reset/accept.hbs"),
    { encoding: "utf8" }
);
const passwordResetAcceptOkHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/password_reset/accept_ok.hbs"),
    { encoding: "utf8" }
);
const passwordResetRejectHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/password_reset/reject.hbs"),
    { encoding: "utf8" }
);
const passwordResetRejectOkHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/password_reset/reject_ok.hbs"),
    { encoding: "utf8" }
);
const emailUpdateRequestHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/email_update/request.hbs"),
    { encoding: "utf8" }
);
const emailUpdateRequestOkHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/email_update/request_ok.hbs"),
    { encoding: "utf8" }
);
const passwordUpdateRequestHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/password_update/request.hbs"),
    { encoding: "utf8" }
);
const passwordUpdateRequestOkHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/password_update/request_ok.hbs"),
    { encoding: "utf8" }
);
const registerRequestHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/register/request.hbs"),
    { encoding: "utf8" }
);
const registerAcceptHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/register/accept.hbs"),
    { encoding: "utf8" }
);
const registerAcceptOkHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/register/accept_ok.hbs"),
    { encoding: "utf8" }
);
const registerRejectHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/register/reject.hbs"),
    { encoding: "utf8" }
);
const registerRejectOkHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/register/reject_ok.hbs"),
    { encoding: "utf8" }
);
const logoutHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/logout.hbs"),
    { encoding: "utf8" }
);
const deleteRequestHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/delete/request.hbs"),
    { encoding: "utf8" }
);
const deleteAcceptHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/delete/accept.hbs"),
    { encoding: "utf8" }
);
const deleteAcceptOkHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/delete/accept_ok.hbs"),
    { encoding: "utf8" }
);
const deleteRejectHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/delete/reject.hbs"),
    { encoding: "utf8" }
);
const deleteRejectOkHbs = fs.readFileSync(
    path.normalize("../sso/http_server/template/delete/reject_ok.hbs"),
    { encoding: "utf8" }
);

const exampleHtml = indexHtml.replace(
    "{{{ content }}}",
    [
        errorHbs,
        authHbs,
        passwordResetRequestHbs,
        passwordResetAcceptHbs,
        passwordResetAcceptOkHbs,
        passwordResetRejectHbs,
        passwordResetRejectOkHbs,
        emailUpdateRequestHbs,
        emailUpdateRequestOkHbs,
        passwordUpdateRequestHbs,
        passwordUpdateRequestOkHbs,
        registerRequestHbs,
        registerAcceptHbs,
        registerAcceptOkHbs,
        registerRejectHbs,
        registerRejectOkHbs,
        logoutHbs,
        deleteRequestHbs,
        deleteAcceptHbs,
        deleteAcceptOkHbs,
        deleteRejectHbs,
        deleteRejectOkHbs,
    ].join("\n")
);

const indexHtmlOut = path.normalize("./tmp/template/index.html");
if (!fs.existsSync("./tmp/template")) {
    fs.mkdirSync("./tmp/template", { recursive: true });
}
fs.writeFileSync(indexHtmlOut, exampleHtml);

proc.execSync("npm run postcss-style");

const styleCss = fs.readFileSync(path.normalize("./tmp/template/style.css"), {
    encoding: "utf8",
});

const templateHtml = minifyHtml(
    indexHtml.replace(
        '<link rel="stylesheet" href="style.css" />',
        `<style>${styleCss}</style>`
    ),
    {
        collapseWhitespace: true,
        minifyCSS: true,
        minifyJS: true,
        removeComments: true,
    }
);

const templateHtmlOut = path.normalize("./tmp/template/template.html");
fs.writeFileSync(templateHtmlOut, templateHtml);
