package main

import (
	"context"
	bot2 "github.com/Light-Keeper/ir-remote/internal/bot"
	"github.com/Light-Keeper/ir-remote/internal/irremote"
	"github.com/Light-Keeper/ir-remote/internal/irremote/encoder"
	"github.com/Light-Keeper/ir-remote/internal/irremote/transport"
	"log"
	"net"
	"os"
	"os/signal"
	"strconv"
	"sync"
	"syscall"
	"time"
)

var udpListenIp = mustGetEnvString("IR_LISTEN_IP")
var irListenUdpPort = mustGetEnvInt("IR_LISTEN_PORT")
var irSharedSecret = mustGetEnvString("IR_SHARED_SECRET")
var botApiKey = mustGetEnvString("BOT_API")
var botAuthorizedUsers = mustGetEnvString("BOT_AUTHORIZED_USERS")

func main() {
	// aesEncoder := encoder.NewAesEncoder(irSharedSecret)
	dummyEncoder := encoder.NewDummyEncoder()
	udp := transport.NewUdpTransport()
	session := irremote.NewSession(udp, dummyEncoder)
	bot := bot2.NewBot(botApiKey, botAuthorizedUsers, session)

	ctx, teardownApp := context.WithCancel(context.Background())

	wg := &sync.WaitGroup{}
	wg.Add(3)

	go func() {
		defer wg.Done()
		err := udp.ListenAndServe(ctx, &net.UDPAddr{
			IP:   net.ParseIP(udpListenIp),
			Port: irListenUdpPort,
		})
		if err != nil {
			panic(err)
		}
	}()

	go func() {
		defer wg.Done()
		session.RunSession(ctx)
	}()

	go func() {
		defer wg.Done()
		err := bot.Run(ctx)
		if err != nil {
			panic(err)
		}
	}()

	// graceful shutdown
	signals := make(chan os.Signal, 1)
	signal.Notify(signals, os.Interrupt, syscall.SIGTERM)
	s := <-signals
	log.Println("Got signal:", s)
	teardownApp()
	if waitTimeout(wg, 5*time.Second) {
		log.Println("Timed out waiting for wait group")
	}
}

func mustGetEnvInt(key string) int {
	val, err := strconv.Atoi(os.Getenv(key))
	assertNoError(err)
	return val
}

func mustGetEnvString(key string) string {
	val := os.Getenv(key)
	if val == "" {
		panic("Missing required environment variable: " + key)
	}
	return val
}

func assertNoError(err error) {
	if err != nil {
		panic(err)
	}
}

// waitTimeout waits for the waitgroup for the specified max timeout.
// Returns true if waiting timed out.
func waitTimeout(wg *sync.WaitGroup, timeout time.Duration) bool {
	c := make(chan struct{})
	go func() {
		defer close(c)
		wg.Wait()
	}()
	select {
	case <-c:
		return false // completed normally
	case <-time.After(timeout):
		return true // timed out
	}
}
