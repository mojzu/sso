import localVarRequest from 'request';

export * from './requestAccessDelete';
export * from './requestAccessUpdate';
export * from './requestApiKeyCreate';
export * from './requestApiKeyDelete';
export * from './requestApiKeyRead';
export * from './requestApiKeyUpdate';
export * from './requestApiKeyVerify';
export * from './requestAuditCreate';
export * from './requestAuditRead';
export * from './requestAuditReadSeek';
export * from './requestCsrf';
export * from './requestOauth2Introspect';
export * from './requestOauth2Token';
export * from './requestUserAccessRead';
export * from './requestUserCreate';
export * from './requestUserCreatePassword';
export * from './requestUserDelete';
export * from './requestUserRead';
export * from './requestUserUpdate';
export * from './requestUserUpdateAccess';
export * from './requestUserUpdatePassword';
export * from './responseAccess';
export * from './responseAccessMany';
export * from './responseAccessManyData';
export * from './responseApiKey';
export * from './responseApiKeyMany';
export * from './responseApiKeyManyData';
export * from './responseAudit';
export * from './responseAuditMany';
export * from './responseAuditManyData';
export * from './responseClient';
export * from './responseCsrf';
export * from './responseUser';
export * from './responseUserMany';
export * from './responseUserManyData';
export * from './responseUserOauth2Provider';
export * from './responseUserPassword';

import * as fs from 'fs';

export interface RequestDetailedFile {
    value: Buffer;
    options?: {
        filename?: string;
        contentType?: string;
    }
}

export type RequestFile = string | Buffer | fs.ReadStream | RequestDetailedFile;


import { RequestAccessDelete } from './requestAccessDelete';
import { RequestAccessUpdate } from './requestAccessUpdate';
import { RequestApiKeyCreate } from './requestApiKeyCreate';
import { RequestApiKeyDelete } from './requestApiKeyDelete';
import { RequestApiKeyRead } from './requestApiKeyRead';
import { RequestApiKeyUpdate } from './requestApiKeyUpdate';
import { RequestApiKeyVerify } from './requestApiKeyVerify';
import { RequestAuditCreate } from './requestAuditCreate';
import { RequestAuditRead } from './requestAuditRead';
import { RequestAuditReadSeek } from './requestAuditReadSeek';
import { RequestCsrf } from './requestCsrf';
import { RequestOauth2Introspect } from './requestOauth2Introspect';
import { RequestOauth2Token } from './requestOauth2Token';
import { RequestUserAccessRead } from './requestUserAccessRead';
import { RequestUserCreate } from './requestUserCreate';
import { RequestUserCreatePassword } from './requestUserCreatePassword';
import { RequestUserDelete } from './requestUserDelete';
import { RequestUserRead } from './requestUserRead';
import { RequestUserUpdate } from './requestUserUpdate';
import { RequestUserUpdateAccess } from './requestUserUpdateAccess';
import { RequestUserUpdatePassword } from './requestUserUpdatePassword';
import { ResponseAccess } from './responseAccess';
import { ResponseAccessMany } from './responseAccessMany';
import { ResponseAccessManyData } from './responseAccessManyData';
import { ResponseApiKey } from './responseApiKey';
import { ResponseApiKeyMany } from './responseApiKeyMany';
import { ResponseApiKeyManyData } from './responseApiKeyManyData';
import { ResponseAudit } from './responseAudit';
import { ResponseAuditMany } from './responseAuditMany';
import { ResponseAuditManyData } from './responseAuditManyData';
import { ResponseClient } from './responseClient';
import { ResponseCsrf } from './responseCsrf';
import { ResponseUser } from './responseUser';
import { ResponseUserMany } from './responseUserMany';
import { ResponseUserManyData } from './responseUserManyData';
import { ResponseUserOauth2Provider } from './responseUserOauth2Provider';
import { ResponseUserPassword } from './responseUserPassword';

/* tslint:disable:no-unused-variable */
let primitives = [
                    "string",
                    "boolean",
                    "double",
                    "integer",
                    "long",
                    "float",
                    "number",
                    "any"
                 ];

let enumsMap: {[index: string]: any} = {
}

let typeMap: {[index: string]: any} = {
    "RequestAccessDelete": RequestAccessDelete,
    "RequestAccessUpdate": RequestAccessUpdate,
    "RequestApiKeyCreate": RequestApiKeyCreate,
    "RequestApiKeyDelete": RequestApiKeyDelete,
    "RequestApiKeyRead": RequestApiKeyRead,
    "RequestApiKeyUpdate": RequestApiKeyUpdate,
    "RequestApiKeyVerify": RequestApiKeyVerify,
    "RequestAuditCreate": RequestAuditCreate,
    "RequestAuditRead": RequestAuditRead,
    "RequestAuditReadSeek": RequestAuditReadSeek,
    "RequestCsrf": RequestCsrf,
    "RequestOauth2Introspect": RequestOauth2Introspect,
    "RequestOauth2Token": RequestOauth2Token,
    "RequestUserAccessRead": RequestUserAccessRead,
    "RequestUserCreate": RequestUserCreate,
    "RequestUserCreatePassword": RequestUserCreatePassword,
    "RequestUserDelete": RequestUserDelete,
    "RequestUserRead": RequestUserRead,
    "RequestUserUpdate": RequestUserUpdate,
    "RequestUserUpdateAccess": RequestUserUpdateAccess,
    "RequestUserUpdatePassword": RequestUserUpdatePassword,
    "ResponseAccess": ResponseAccess,
    "ResponseAccessMany": ResponseAccessMany,
    "ResponseAccessManyData": ResponseAccessManyData,
    "ResponseApiKey": ResponseApiKey,
    "ResponseApiKeyMany": ResponseApiKeyMany,
    "ResponseApiKeyManyData": ResponseApiKeyManyData,
    "ResponseAudit": ResponseAudit,
    "ResponseAuditMany": ResponseAuditMany,
    "ResponseAuditManyData": ResponseAuditManyData,
    "ResponseClient": ResponseClient,
    "ResponseCsrf": ResponseCsrf,
    "ResponseUser": ResponseUser,
    "ResponseUserMany": ResponseUserMany,
    "ResponseUserManyData": ResponseUserManyData,
    "ResponseUserOauth2Provider": ResponseUserOauth2Provider,
    "ResponseUserPassword": ResponseUserPassword,
}

export class ObjectSerializer {
    public static findCorrectType(data: any, expectedType: string) {
        if (data == undefined) {
            return expectedType;
        } else if (primitives.indexOf(expectedType.toLowerCase()) !== -1) {
            return expectedType;
        } else if (expectedType === "Date") {
            return expectedType;
        } else {
            if (enumsMap[expectedType]) {
                return expectedType;
            }

            if (!typeMap[expectedType]) {
                return expectedType; // w/e we don't know the type
            }

            // Check the discriminator
            let discriminatorProperty = typeMap[expectedType].discriminator;
            if (discriminatorProperty == null) {
                return expectedType; // the type does not have a discriminator. use it.
            } else {
                if (data[discriminatorProperty]) {
                    var discriminatorType = data[discriminatorProperty];
                    if(typeMap[discriminatorType]){
                        return discriminatorType; // use the type given in the discriminator
                    } else {
                        return expectedType; // discriminator did not map to a type
                    }
                } else {
                    return expectedType; // discriminator was not present (or an empty string)
                }
            }
        }
    }

    public static serialize(data: any, type: string) {
        if (data == undefined) {
            return data;
        } else if (primitives.indexOf(type.toLowerCase()) !== -1) {
            return data;
        } else if (type.lastIndexOf("Array<", 0) === 0) { // string.startsWith pre es6
            let subType: string = type.replace("Array<", ""); // Array<Type> => Type>
            subType = subType.substring(0, subType.length - 1); // Type> => Type
            let transformedData: any[] = [];
            for (let index in data) {
                let date = data[index];
                transformedData.push(ObjectSerializer.serialize(date, subType));
            }
            return transformedData;
        } else if (type === "Date") {
            return data.toISOString();
        } else {
            if (enumsMap[type]) {
                return data;
            }
            if (!typeMap[type]) { // in case we dont know the type
                return data;
            }

            // Get the actual type of this object
            type = this.findCorrectType(data, type);

            // get the map for the correct type.
            let attributeTypes = typeMap[type].getAttributeTypeMap();
            let instance: {[index: string]: any} = {};
            for (let index in attributeTypes) {
                let attributeType = attributeTypes[index];
                instance[attributeType.baseName] = ObjectSerializer.serialize(data[attributeType.name], attributeType.type);
            }
            return instance;
        }
    }

    public static deserialize(data: any, type: string) {
        // polymorphism may change the actual type.
        type = ObjectSerializer.findCorrectType(data, type);
        if (data == undefined) {
            return data;
        } else if (primitives.indexOf(type.toLowerCase()) !== -1) {
            return data;
        } else if (type.lastIndexOf("Array<", 0) === 0) { // string.startsWith pre es6
            let subType: string = type.replace("Array<", ""); // Array<Type> => Type>
            subType = subType.substring(0, subType.length - 1); // Type> => Type
            let transformedData: any[] = [];
            for (let index in data) {
                let date = data[index];
                transformedData.push(ObjectSerializer.deserialize(date, subType));
            }
            return transformedData;
        } else if (type === "Date") {
            return new Date(data);
        } else {
            if (enumsMap[type]) {// is Enum
                return data;
            }

            if (!typeMap[type]) { // dont know the type
                return data;
            }
            let instance = new typeMap[type]();
            let attributeTypes = typeMap[type].getAttributeTypeMap();
            for (let index in attributeTypes) {
                let attributeType = attributeTypes[index];
                instance[attributeType.name] = ObjectSerializer.deserialize(data[attributeType.baseName], attributeType.type);
            }
            return instance;
        }
    }
}

export interface Authentication {
    /**
    * Apply authentication settings to header and query params.
    */
    applyToRequest(requestOptions: localVarRequest.Options): Promise<void> | void;
}

export class HttpBasicAuth implements Authentication {
    public username: string = '';
    public password: string = '';

    applyToRequest(requestOptions: localVarRequest.Options): void {
        requestOptions.auth = {
            username: this.username, password: this.password
        }
    }
}

export class HttpBearerAuth implements Authentication {
    public accessToken: string | (() => string) = '';

    applyToRequest(requestOptions: localVarRequest.Options): void {
        if (requestOptions && requestOptions.headers) {
            const accessToken = typeof this.accessToken === 'function'
                            ? this.accessToken()
                            : this.accessToken;
            requestOptions.headers["Authorization"] = "Bearer " + accessToken;
        }
    }
}

export class ApiKeyAuth implements Authentication {
    public apiKey: string = '';

    constructor(private location: string, private paramName: string) {
    }

    applyToRequest(requestOptions: localVarRequest.Options): void {
        if (this.location == "query") {
            (<any>requestOptions.qs)[this.paramName] = this.apiKey;
        } else if (this.location == "header" && requestOptions && requestOptions.headers) {
            requestOptions.headers[this.paramName] = this.apiKey;
        } else if (this.location == 'cookie' && requestOptions && requestOptions.headers) {
            if (requestOptions.headers['Cookie']) {
                requestOptions.headers['Cookie'] += '; ' + this.paramName + '=' + encodeURIComponent(this.apiKey);
            }
            else {
                requestOptions.headers['Cookie'] = this.paramName + '=' + encodeURIComponent(this.apiKey);
            }
        }
    }
}

export class OAuth implements Authentication {
    public accessToken: string = '';

    applyToRequest(requestOptions: localVarRequest.Options): void {
        if (requestOptions && requestOptions.headers) {
            requestOptions.headers["Authorization"] = "Bearer " + this.accessToken;
        }
    }
}

export class VoidAuth implements Authentication {
    public username: string = '';
    public password: string = '';

    applyToRequest(_: localVarRequest.Options): void {
        // Do nothing
    }
}

export type Interceptor = (requestOptions: localVarRequest.Options) => (Promise<void> | void);
