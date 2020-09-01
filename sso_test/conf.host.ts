import { Config } from "protractor";

export let config: Config = {
    framework: "jasmine",
    capabilities: {
        browserName: "firefox",
    },
    specs: ["test/host.js"],
    seleniumAddress: "http://localhost:4444/wd/hub",
};
