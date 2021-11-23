package main

import (
	"cloud.google.com/go/spanner"
	"context"
	"fmt"
	"github.com/go-chi/chi/v5"
	"go-api/cmd/internal"
	"net/http"
	"os"
	"os/signal"
	"syscall"
)

func main() {
	router := chi.NewRouter()

	spannerDSN := os.Getenv("SPANNER_DSN")
	if len(spannerDSN) == 0 {
		panic("SPANNER_DSN is required")
	}

	client, err := spanner.NewClient(context.Background(), spannerDSN)
	if err != nil {
		panic(err)
	}
	defer client.Close()

	router.Post("/CreateUser", internal.CreateUser(client))
	router.Post("/ReadOnly", internal.ReadInventory(client))
	router.Post("/ReadWrite", internal.UpdateInventory(client))
	server := http.Server{Addr: ":3032", Handler: router}
	go func() {
		sigint := make(chan os.Signal, 1)
		signal.Notify(sigint, os.Interrupt)
		signal.Notify(sigint, syscall.SIGTERM)
		<-sigint
		fmt.Println("Shutdown server.")
		_ = server.Shutdown(context.Background())
	}()

	fmt.Println("Listening on http://0.0.0.0:3032")
	if err = server.ListenAndServe(); err != http.ErrServerClosed {
		fmt.Printf("err %+v\n", err)
	}
}