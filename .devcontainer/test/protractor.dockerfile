FROM sso/build-test:latest

ENTRYPOINT ["npm", "run", "protractor", "sso_test/tmp/conf.docker.js"]
