import { Config } from "protractor";

export let config: Config = {
    framework: "jasmine",
    multiCapabilities: [
        // todo: Fix firefox gets errors when running in test-ci container
        // {
        //     browserName: "firefox",
        // },
        {
            browserName: "chrome",
        },
    ],
    specs: ["test/api.js", "test/browser.js"],
    seleniumAddress: "http://selenium:4444/wd/hub",
};
