package encoder

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

type dummy2 struct {
	Number int
}

func TestDummyEncodeCommand(t *testing.T) {
	encoder := NewDummyEncoder()
	x := dummy{Number: 42}

	encrypted := encoder.Encrypt(x)

	cmd := dummy{}
	err := encoder.Decrypt(encrypted, &cmd)
	assert.NoError(t, err)
	assert.Equal(t, x, cmd)
}
