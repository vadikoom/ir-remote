package transport

import (
	"context"
	"github.com/stretchr/testify/assert"
	"net"
	"testing"
)

func TestIntegration_Udp(t *testing.T) {
	udp := NewUdpTransport()
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	go func() {
		cancel()
	}()

	err := udp.ListenAndServe(ctx, &net.UDPAddr{
		IP:   net.IPv4(127, 0, 0, 1),
		Port: 1234,
	})
	assert.NoError(t, err)
}
