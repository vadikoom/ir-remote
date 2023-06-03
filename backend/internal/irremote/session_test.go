package irremote

import (
	"context"
	"github.com/Light-Keeper/ir-remote/internal/irremote/encoder"
	"github.com/Light-Keeper/ir-remote/internal/irremote/transport"
	"github.com/davecgh/go-spew/spew"
	"net"
	"testing"
	"time"
)

func TestSession_Integration(t *testing.T) {
	dummyEncoder := encoder.NewDummyEncoder()
	udpTransport := transport.NewUdpTransport()
	removeDevice := transport.NewUdpTransport()
	session := NewSession(udpTransport, dummyEncoder)

	// Start the session
	cxt, cancel := context.WithCancel(context.Background())

	go func() {
		println("Running udp transport")
		err := udpTransport.ListenAndServe(cxt, &net.UDPAddr{
			IP:   net.IPv4(127, 0, 0, 1),
			Port: 1234,
		})
		if err != nil {
			panic(err)
		}
	}()

	go func() {
		println("Running udp transport for remote device")
		err := removeDevice.ListenAndServe(cxt, &net.UDPAddr{
			IP:   net.IPv4(127, 0, 0, 1),
			Port: 1235,
		})
		if err != nil {
			panic(err)
		}
	}()

	go func() {
		println("Running session")
		session.RunSession(cxt)
	}()

	// listen for incoming commands, pretend to be a remote device
	go func() {
		// we should start
		err := removeDevice.Send(transport.UdpPacket{
			Addr: &net.UDPAddr{
				IP:   net.IPv4(127, 0, 0, 1),
				Port: 1234,
			},
			Data: pack(Status{LastCommandSequenceNumber: 0}),
		})
		println("Sent initial package ", err)

		for {
			select {
			case <-cxt.Done():
				return
			case packet := <-removeDevice.Receive():
				println("Received command")
				spew.Dump(packet)
				println("Sending confirmation")
				err := removeDevice.Send(transport.UdpPacket{
					Addr: &net.UDPAddr{
						IP:   net.IPv4(127, 0, 0, 1),
						Port: 1234,
					},
					Data: pack(Status{LastCommandSequenceNumber: 10}),
				})
				println("Confirmation sent, result: ", err)
			}
		}
	}()

	// send command once in a while
	go func() {
		for {
			<-time.After(2 * time.Second)
			println("Sending command")
			err := session.SendCommand(cxt, []int{1, 2, 3, 4, 5, 6, 7, 8})
			print("Command sent, result: ")
			spew.Dump(err)
		}
	}()

	<-time.After(1 * time.Hour)
	cancel()
}

func pack(cmd any) []byte {
	return encoder.NewDummyEncoder().Encrypt(cmd)
}
