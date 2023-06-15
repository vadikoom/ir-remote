package bot

import (
	"github.com/davecgh/go-spew/spew"
	"testing"
)

func TestConvertCommands(t *testing.T) {
	buf := commandOff[2 : len(commandOff)-1]
	bits := make([]byte, len(buf))

	for i := 0; i < len(buf); i += 2 {
		if inRange(buf[i], 300, 700) && inRange(buf[i+1], 300, 700) {
			bits[i/2] = 0
		} else if inRange(buf[i], 300, 700) && inRange(buf[i+1], 1500, 1800) {
			bits[i/2] = 1
		} else {
			t.Errorf("invalid value: %d %d", buf[i], buf[i+1])
		}

	}
	spew.Dump(len(bits))
}

func inRange(v int, min int, max int) bool {
	return v >= min && v <= max
}
