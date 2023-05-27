package main

import (
	"fmt"
	"net/http"
)

func main() {
	startHelloworldServer()
}

func startHelloworldServer() {
	// Define the HTTP route and handler function
	http.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintf(w, "Hello, world!")
	})

	// Start the HTTP server on port 8000
	fmt.Println("Staring server on http://localhost:8000")

	err := http.ListenAndServe("0.0.0.0:8000", nil)
	if err != nil {
		fmt.Println("Error starting server: ", err)
		return
	}
}
