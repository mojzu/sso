# depend: docker pull node:14.15-buster
FROM node:14.15-buster

WORKDIR /build

COPY ./package.json /build/package.json
COPY ./package-lock.json /build/package-lock.json
RUN npm install

COPY ./sso_test /build/sso_test
RUN npm run tsc
