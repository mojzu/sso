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
    seleniumAddress: "http://localhost:4444/wd/hub",
};
