import { browser } from "protractor";
import "jasmine";
import { api, browser_sleep } from "./util";

describe("sso-host", function () {
    beforeEach(async function () {
        browser.waitForAngularEnabled(false);
    });

    it("should run", async function () {
        // fix: Fixes must run all tests inside docker container, can develop tests here
        await browser_sleep(5);
    });
});
