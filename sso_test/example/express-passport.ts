const express = require("express");
const session = require("express-session");
const passport = require("passport");
const OAuth2Strategy = require("passport-oauth2");

export interface Config {
    clientId: string;
    clientSecret: string;
    authorizeUri: string;
    tokenUri: string;
    redirectUri: string;
    port: number;
    authUri: string;
}

export function app_listen(config: Config) {
    const app = express();

    // fix: Fixes client does not support basic authentication
    const basicAuth = `Basic ${new Buffer(
        config.clientId + ":" + config.clientSecret
    ).toString("base64")}`;

    passport.use(
        new OAuth2Strategy(
            {
                authorizationURL: config.authorizeUri,
                tokenURL: config.tokenUri,
                clientID: config.clientId,
                clientSecret: config.clientSecret,
                callbackURL: config.redirectUri,
                customHeaders: {
                    Authorization: basicAuth,
                },
                state: true,
            },
            function (accessToken, refreshToken, profile, cb) {
                return cb(undefined, {
                    profile,
                    accessToken,
                    refreshToken,
                });
            }
        )
    );
    passport.serializeUser(function (user, done) {
        done(null, user);
    });
    passport.deserializeUser(function (user, done) {
        done(null, user);
    });

    app.use(
        session({
            secret: "2M7j6sN3Vjd2RtrlSYQ9V+5PZdK0DVY6dGWlw+qd25A=",
            resave: false,
            saveUninitialized: false,
            cookie: { secure: false },
        })
    );
    app.use(passport.initialize());
    app.use(passport.session());

    app.get("/oauth2", passport.authenticate("oauth2"), function (req, res) {
        res.redirect("/");
    });

    app.get("/", function (req, res) {
        if (req.user == null) {
            res.redirect("/oauth2");
        } else {
            let emailUpdate = encodeURI(
                `${config.authUri}/email-update?client_id=${config.clientId}&redirect_uri=${config.redirectUri}`
            );
            let passwordUpdate = encodeURI(
                `${config.authUri}/password-update?client_id=${config.clientId}&redirect_uri=${config.redirectUri}`
            );
            let userLogout = encodeURI(
                `${config.authUri}/logout?client_id=${config.clientId}&redirect_uri=${config.redirectUri}`
            );
            let userDelete = encodeURI(
                `${config.authUri}/delete?client_id=${config.clientId}&redirect_uri=${config.redirectUri}`
            );
            res.send(
                `<pre>user ${JSON.stringify(req.user, null, 4)}</pre>
                <div><a href="${emailUpdate}">Email Update</a></div>
                <div><a href="${passwordUpdate}">Password Update</a></div>
                <div><a href="${userLogout}">Logout</a></div>
                <div><a href="${userDelete}">Delete User</a></div>
                <div id="access-token">${req.user.accessToken}</div>
                <div id="refresh-token">${req.user.refreshToken}</div>`
            );
        }
    });

    app.listen(config.port, () => {
        console.log(`listening at http://localhost:${config.port}`);
    });

    return app;
}
