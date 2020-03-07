package main

import (
	"flag"
	"golang.org/x/net/context"
	"net/http"
	"os"
	"strings"

	"github.com/golang/glog"
	"github.com/grpc-ecosystem/grpc-gateway/runtime"
	"google.golang.org/grpc"
)

var (
	// command-line options:
	// gRPC server endpoint
	grpcServerEndpoint = flag.String("grpc-server-url", os.Getenv("SSO_GRPC_URL"), "gRPC server URL")
	// CORS allow origins
	corsAllowOrigin = flag.String("cors-allow-origin", os.Getenv("SSO_CORS_ALLOW_ORIGIN"), "CORS allow origins")
)

func run() error {
	ctx := context.Background()
	ctx, cancel := context.WithCancel(ctx)
	defer cancel()

	// Register gRPC server endpoint
	// Note: Make sure the gRPC server is running properly and accessible
	mux := runtime.NewServeMux()
	opts := []grpc.DialOption{grpc.WithInsecure()}
	err := RegisterSsoHandlerFromEndpoint(ctx, mux, *grpcServerEndpoint, opts)
	if err != nil {
		return err
	}

    // CORS allow origin array.
	corsAllowOriginArray := sliceDeleteEmptyString(strings.Split(*corsAllowOrigin, ","))

	// Start HTTP server (and proxy calls to gRPC server endpoint)
	return http.ListenAndServe(":8042", corsHandler(mux, corsAllowOriginArray))
}

func corsHandler(h http.Handler, corsAllowOrigin []string) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if len(corsAllowOrigin) > 0 {
			origin := r.Header.Get("Origin")
			if origin != "" && sliceStringIn(origin, corsAllowOrigin) {
				// Origin defined and in allow list.
				// Set allow origin header, handle preflight requests.
				w.Header().Set("Access-Control-Allow-Origin", origin)
				if r.Method == "OPTIONS" {
					corsPreflightHandler(w, r)
					return
				}
			} else {
				// Origin not defined or not allowed, discard request.
				w.WriteHeader(http.StatusNoContent)
				return
			}
		} else {
			// CORS not configured, allow by default, handle preflight requests.
			w.Header().Set("Access-Control-Allow-Origin", "*")
			if r.Method == "OPTIONS" {
				corsPreflightHandler(w, r)
				return
			}
		}
		h.ServeHTTP(w, r)
	})
}

func corsPreflightHandler(w http.ResponseWriter, r *http.Request) {
	headers := []string{"User-Agent", "Content-Type", "Accept", "Authorization"}
	w.Header().Set("Access-Control-Allow-Headers", strings.Join(headers, ","))
	methods := []string{"GET", "POST", "PATCH", "DELETE"}
	w.Header().Set("Access-Control-Allow-Methods", strings.Join(methods, ","))
	glog.Infof("preflight request for %s", r.URL.Path)
	w.WriteHeader(http.StatusNoContent)
}

func sliceDeleteEmptyString(s []string) []string {
	var r []string
	for _, str := range s {
		if str != "" {
			r = append(r, str)
		}
	}
	return r
}

func sliceStringIn(a string, list []string) bool {
	for _, b := range list {
		if b == a {
			return true
		}
	}
	return false
}

func main() {
	flag.Parse()
	defer glog.Flush()

	if err := run(); err != nil {
		glog.Fatal(err)
	}
}
