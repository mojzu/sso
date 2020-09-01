import { app_listen } from "./express-passport";
import {
    AUTHORIZE_URI,
    TOKEN_URI,
    CLIENT_ID,
    REDIRECT_URI,
    CLIENT_SECRET,
    AUTH_URI,
} from "../test/util";

app_listen({
    clientId: CLIENT_ID,
    clientSecret: CLIENT_SECRET,
    authorizeUri: AUTHORIZE_URI,
    tokenUri: TOKEN_URI,
    redirectUri: REDIRECT_URI,
    port: 8080,
    authUri: AUTH_URI,
});
