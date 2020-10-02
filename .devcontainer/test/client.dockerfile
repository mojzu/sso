FROM sso/build-test:latest

COPY .devcontainer/build/scripts/wait-for-it.sh /wait-for-it.sh
RUN chmod +x /wait-for-it.sh

ENTRYPOINT ["node", "/opt/sso_test/tmp/example/express-passport-docker.js"]
