package webserver

import (
	"context"
	"encoding/json"
	"fmt"
	"github.com/Light-Keeper/ir-remote/internal/irremote"
	"github.com/go-playground/validator/v10"
	"log"
	"net/http"
	"time"
)

type WebServer interface {
	ListenAndServe(ctx context.Context) error
}

func NewWebServer(httpPort int, httpListenIp string, remote *irremote.Session) WebServer {
	return &webServer{
		httpPort:     httpPort,
		httpListenIp: httpListenIp,
		remote:       remote,
	}
}

type webServer struct {
	httpPort     int
	httpListenIp string
	remote       *irremote.Session
	svr          *http.Server
}

func (s *webServer) ListenAndServe(ctx context.Context) error {
	address := fmt.Sprintf("%v:%v", s.httpListenIp, s.httpPort)
	s.svr = &http.Server{Addr: address}

	mux := http.NewServeMux()
	mux.Handle("/command", logRequestHandler(s.ServeCommand()))
	mux.Handle("/status", logRequestHandler(s.ServeStatus()))
	s.svr.Handler = mux

	go func() {
		log.Printf("starting web server on %v", address)
		// always returns error. ErrServerClosed on graceful close
		if err := s.svr.ListenAndServe(); err != http.ErrServerClosed {
			log.Println("error starting web server: ", err.Error())
		}
	}()

	<-ctx.Done()
	timeout, cancelFunc := context.WithTimeout(context.Background(), 2*time.Second)
	defer cancelFunc()
	return s.svr.Shutdown(timeout)
}

type Command struct {
	Intervals []int `json:"intervals" validate:"required,min=2,max=300"`
}

var validatorInstance = validator.New()

func (s *webServer) ServeCommand() http.HandlerFunc {
	return func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != http.MethodPost {
			respond(http.StatusMethodNotAllowed, map[string]string{"error": "method not allowed"}, writer)
			return
		}

		var command Command

		if err := json.NewDecoder(request.Body).Decode(&command); err != nil {
			respond(http.StatusBadRequest, map[string]string{"error": err.Error()}, writer)
			return
		}

		if err := validatorInstance.Struct(command); err != nil {
			respond(http.StatusBadRequest, map[string]string{"error": err.Error()}, writer)
			return
		}

		if err := s.remote.SendCommand(context.Background(), command.Intervals); err != nil {
			respond(http.StatusInternalServerError, map[string]string{"error": err.Error()}, writer)
			return
		}

		respond(http.StatusOK, map[string]string{"status": "ok"}, writer)
	}
}

func (s *webServer) ServeStatus() http.HandlerFunc {
	return func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != http.MethodGet {
			respond(http.StatusMethodNotAllowed, map[string]string{"error": "method not allowed"}, writer)
			return
		}

		respond(http.StatusOK, map[string]bool{"online": s.remote.IsOnline()}, writer)
	}
}

func logRequestHandler(h http.Handler) http.Handler {
	fn := func(w http.ResponseWriter, r *http.Request) {
		// call the original http.Handler we're wrapping
		h.ServeHTTP(w, r)

		// gather information about request and log it
		uri := r.URL.String()
		method := r.Method
		// ... more information
		log.Printf("%v %v", method, uri)
	}

	// http.HandlerFunc wraps a function so that it
	// implements http.Handler interface
	return http.HandlerFunc(fn)
}

func respond(status int, obj any, writer http.ResponseWriter) {
	writer.WriteHeader(status)
	marshal, err := json.Marshal(obj)
	if err != nil {
		log.Println("error marshalling error")
		return
	}
	writer.Write(marshal)
}
