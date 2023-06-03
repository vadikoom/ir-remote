package encoder

type Encoder interface {
	Encrypt(message any) []byte
	Decrypt(data []byte, into any) error
}
