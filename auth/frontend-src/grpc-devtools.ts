// https://github.com/soc221b/grpc-devtools
import type { Interceptor } from "@connectrpc/connect";

declare const __gRPC_devtools__:
    | undefined
    | {
          connectEsInterceptor: Interceptor;
      };

export const interceptors =
    typeof __gRPC_devtools__ === "object"
        ? [__gRPC_devtools__.connectEsInterceptor]
        : [];
