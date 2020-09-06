import { app_listen } from "./express-passport";
import {
    AUTHORIZE_URI,
    CLIENT_ID,
    REDIRECT_URI,
    CLIENT_SECRET,
    AUTH_URI,
} from "../test/util";

app_listen({
    clientId: CLIENT_ID,
    clientSecret: CLIENT_SECRET,
    authorizeUri: AUTHORIZE_URI,
    tokenUri: `http://sso:7042/v2/oauth2/token`,
    redirectUri: REDIRECT_URI,
    port: 8080,
    authUri: AUTH_URI,
});
