package transport

import "net"

type UdpPacket struct {
	Addr *net.UDPAddr
	Data []byte
}

type Transport interface {
	Send(packet UdpPacket) error
	Receive() <-chan UdpPacket
}
