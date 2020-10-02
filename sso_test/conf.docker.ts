import { Config } from "protractor";

export let config: Config = {
    framework: "jasmine",
    multiCapabilities: [
        {
            browserName: "firefox",
        },
        {
            browserName: "chrome",
        },
    ],
    specs: ["test/api.js", "test/browser.js"],
    // todo: Fix running tests using selenium docker container (localhost address refactoring?)
    // seleniumAddress: "http://selenium:6444/wd/hub",
    seleniumAddress: "http://localhost:4444/wd/hub",
};
