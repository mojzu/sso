FROM sso/build-test:latest

ENTRYPOINT ["node", "/opt/sso_test/tmp/example/express-passport-docker.js"]
