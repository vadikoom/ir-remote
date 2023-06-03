package encoder

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

type dummy struct {
	Number int
}

func TestEncodeCommand(t *testing.T) {
	encoder := NewAesEncoder("my super secret key")
	x := dummy{Number: 42}

	encrypted := encoder.Encrypt(x)

	cmd := dummy{}
	err := encoder.Decrypt(encrypted, &cmd)
	assert.NoError(t, err)
	assert.Equal(t, x, cmd)
}

func TestEncodeStatus(t *testing.T) {
	encoder := NewAesEncoder("my super secret key")
	x := dummy{Number: 42}

	encrypted := encoder.Encrypt(x)

	status := dummy{}
	err := encoder.Decrypt(encrypted, &status)
	assert.NoError(t, err)
	assert.Equal(t, x, status)
}

func TestEncoder_DecryptInvalid(t *testing.T) {
	encoder := NewAesEncoder("my super secret key")
	err := encoder.Decrypt([]byte{0x01, 0x02, 0x03}, &dummy{})
	assert.Contains(t, err.Error(), "invalid block size")

	err = encoder.Decrypt([]byte{
		0x01, 0x02, 0x03, 0x04, 0x01, 0x02, 0x03, 0x04, 0x01, 0x02, 0x03, 0x04, 0x01, 0x02, 0x03, 0x04, // 16 bytes iv
		0x01, 0x02, 0x03, 0x04, 0x01, 0x02, 0x03, 0x04, 0x01, 0x02, 0x03, 0x04, 0x01, 0x02, 0x03, 0x04, // 16 bytes random data
	}, &dummy{})

	assert.Contains(t, err.Error(), "invalid magic")
}
