package transport

import (
	"context"
	"fmt"
	"log"
	"net"
	"sync"
)

type UdpTransport struct {
	conn      *net.UDPConn
	receive   chan UdpPacket
	readiness chan struct{}
}

func NewUdpTransport() *UdpTransport {
	return &UdpTransport{
		receive:   make(chan UdpPacket, 10),
		readiness: make(chan struct{}),
	}
}

func (t *UdpTransport) ListenAndServe(ctx context.Context, addr *net.UDPAddr) error {
	log.Println("Listening on", addr)
	conn, err := net.ListenUDP("udp", addr)
	if err != nil {
		return err
	}

	log.Println("Successfully started listener ", conn.LocalAddr())

	t.conn = conn

	wg := sync.WaitGroup{}
	wg.Add(1)

	go func() {
		defer wg.Done()
		// ready to send and receive
		close(t.readiness)

		for {
			buf := make([]byte, 1024)
			n, addr, err := t.conn.ReadFromUDP(buf)
			if err != nil {
				if ctx.Err() != nil {
					// gracefully shutdown
					break
				} else {
					// is this the right way to handle this?
					// let it crash for now
					log.Fatal("Error reading from UDP:", err)
				}
			}

			log.Println("Received", n, "bytes from", addr)

			t.receive <- UdpPacket{
				Addr: addr,
				Data: buf[:n],
			}
		}
	}()

	<-ctx.Done()
	err = t.conn.Close()
	if err != nil {
		return err
	}
	wg.Wait()
	return nil
}

func (t *UdpTransport) Send(packet UdpPacket) error {
	<-t.readiness
	log.Println("Sending", len(packet.Data), "bytes to", packet.Addr)

	n, err := t.conn.WriteToUDP(packet.Data, packet.Addr)
	if err != nil {
		return err
	}
	if n != len(packet.Data) {
		return fmt.Errorf("wrote %d bytes, expected %d", n, len(packet.Data))
	}
	return nil
}

func (t *UdpTransport) Receive() <-chan UdpPacket {
	return t.receive
}
