package encoder

import (
	"crypto/aes"
	"crypto/rand"
	"crypto/sha256"
	"encoding/json"
	"errors"
	"io"
)

const Magic = "IRREMOTE"

type aesEncoder struct {
	sharedSecret [32]byte
}

func NewAesEncoder(sharedSecret string) Encoder {
	return &aesEncoder{
		sharedSecret: sha256.Sum256([]byte(sharedSecret)),
	}
}

func (e *aesEncoder) Encrypt(command any) []byte {
	jsonBytes, err := json.Marshal(command)

	if err != nil {
		panic(err)
	}

	padding := aes.BlockSize - ((len(Magic) + len(jsonBytes)) % aes.BlockSize)
	buf := make([]byte, aes.BlockSize+len(Magic)+len(jsonBytes)+padding)

	if _, err := io.ReadFull(rand.Reader, buf[:aes.BlockSize]); err != nil {
		panic(err)
	}

	copy(buf[aes.BlockSize:], Magic)
	copy(buf[aes.BlockSize+len(Magic):], jsonBytes)
	for i := aes.BlockSize + len(Magic) + len(jsonBytes); i < len(buf); i++ {
		buf[i] = byte(padding)
	}

	cipher, err := aes.NewCipher(e.sharedSecret[:])
	if err != nil {
		panic(err)
	}

	for i := 0; i < len(buf); i += aes.BlockSize {
		cipher.Encrypt(buf[i:], buf[i:])
	}

	return buf
}

func (e *aesEncoder) Decrypt(data []byte, into any) error {
	cipher, err := aes.NewCipher(e.sharedSecret[:])
	if err != nil {
		panic(err)
	}

	if len(data)%aes.BlockSize != 0 || len(data) < 2*aes.BlockSize {
		return errors.New("invalid block size")
	}

	for i := 0; i < len(data); i += aes.BlockSize {
		cipher.Decrypt(data[i:], data[i:])
	}

	padding := data[len(data)-1]

	if padding > aes.BlockSize || padding == 0 {
		return errors.New("invalid padding")
	}

	// remove iv and padding
	data = data[aes.BlockSize : len(data)-int(padding)]

	if string(data[:len(Magic)]) != Magic {
		return errors.New("invalid magic")
	}

	data = data[len(Magic):]
	return json.Unmarshal(data, into)
}
