package irremote

import (
	"context"
	"errors"
	"github.com/Light-Keeper/ir-remote/internal/irremote/encoder"
	"github.com/Light-Keeper/ir-remote/internal/irremote/transport"
	"log"
	"net"
	"sync"
	"time"
)

const ExpectedPingInterval = 10

type Session struct {
	lastKnownRemoteAddress *net.UDPAddr
	lastTimeSeen           int64
	lastCommandNumber      int64

	netLayer transport.Transport
	encoder  encoder.Encoder

	mx                     sync.Mutex
	remoteMessageBroadcast map[int64]chan Status
}

func NewSession(netLayer transport.Transport, encoder encoder.Encoder) *Session {
	return &Session{
		netLayer:               netLayer,
		encoder:                encoder,
		remoteMessageBroadcast: make(map[int64]chan Status),
	}
}

func (s *Session) RunSession(ctx context.Context) {
	for {
		select {
		case <-ctx.Done():
			return

		case msg := <-s.netLayer.Receive():
			s.onRemoteMessage(ctx, msg)
		}
	}
}

func (s *Session) IsOnline() bool {
	s.mx.Lock()
	defer s.mx.Unlock()
	return s.lastKnownRemoteAddress != nil && time.Now().Unix()-s.lastTimeSeen < 3*ExpectedPingInterval
}

func (s *Session) SendCommand(ctx context.Context, cmdBytes []int) error {
	if !s.IsOnline() {
		return errors.New("session is offline")
	}
	onUpdate := make(chan Status, 10)

	var addr *net.UDPAddr
	var cmd Command

	func() {
		s.mx.Lock()
		defer s.mx.Unlock()
		s.lastCommandNumber++
		cmd = Command{
			Data:           cmdBytes,
			SequenceNumber: s.lastCommandNumber,
		}
		addr = s.lastKnownRemoteAddress
		s.remoteMessageBroadcast[cmd.SequenceNumber] = onUpdate
	}()

	defer func() {
		s.mx.Lock()
		defer s.mx.Unlock()
		delete(s.remoteMessageBroadcast, cmd.SequenceNumber)
	}()

	packet := transport.UdpPacket{
		Addr: addr,
		Data: s.encoder.Encrypt(cmd),
	}

	attempts := 10

	for {
		attempts--
		if attempts == 0 {
			return errors.New("failed to send command, no response from remote")
		}
		err := s.netLayer.Send(packet)
		if err != nil {
			return err
		}

		select {
		case <-ctx.Done():
			return ctx.Err()

		case <-time.After(1 * time.Second):
			continue

		case s := <-onUpdate:
			if s.LastCommandSequenceNumber >= cmd.SequenceNumber {
				return nil
			} else {
				continue
			}
		}
	}
}

func (s *Session) onRemoteMessage(ctx context.Context, msg transport.UdpPacket) {
	status := Status{}
	err := s.encoder.Decrypt(msg.Data, &status)
	if err != nil {
		log.Println("failed to decrypt message", err)
		return
	}

	var notify []chan Status
	func() {
		s.mx.Lock()
		defer s.mx.Unlock()

		s.lastKnownRemoteAddress = msg.Addr
		s.lastTimeSeen = time.Now().Unix()
		if status.LastCommandSequenceNumber > s.lastCommandNumber {
			s.lastCommandNumber = status.LastCommandSequenceNumber
		}

		notify = make([]chan Status, 0, len(s.remoteMessageBroadcast))
		for _, ch := range s.remoteMessageBroadcast {
			notify = append(notify, ch)
		}
	}()

	for _, ch := range notify {
		select {
		case <-ctx.Done():
			return
		case ch <- status:
		default:
		}
	}
}
