import {
    create,
    toJson,
    toJsonString,
    fromJsonString,
} from "@bufbuild/protobuf";
import {
    AuthService,
    ListUsersResponseSchema,
} from "../../generated-proto/js/auth/v2/auth_pb";
import {
    GetCSRFTokenRequestSchema,
    GetCSRFTokenResponseSchema,
} from "../../generated-proto/js/auth/v2/jwt_pb";
import {
    createConnectTransport,
    createGrpcWebTransport,
} from "@connectrpc/connect-web";
import { createClient } from "@connectrpc/connect";
import { interceptors } from "./grpc-devtools";

const transport = createConnectTransport({
    baseUrl: "/",
    interceptors: [...interceptors],
});
const grpcClient = createClient(AuthService, transport);
// @ts-ignore
window.grpcClient = grpcClient;

async function main() {
    let token = await grpcClient
        .getCSRFToken({})
        .then(toJson.bind(null, GetCSRFTokenResponseSchema));

    let users = await grpcClient
        .listUsers({})
        .then(toJson.bind(null, ListUsersResponseSchema));
    console.log(users);
}

main();
