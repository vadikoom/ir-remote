package commands

import (
	"errors"
	"fmt"
	"reflect"
)

const NEC_SHORT = 562
const NEC_LONG = 1687

const NEC_INITIATOR = 4300
const NEC_FILLER = 9000 - NEC_INITIATOR

type NecChainedCommand struct {
	cmd [3]byte
}

var _ Command = &NecChainedCommand{}

// ParseFromSignalSequence parses a signal sequence into a command
func (n *NecChainedCommand) ParseFromSignalSequence(signalSequence []int) error {
	listsOfBits := make([][]int, 0)
	currentList := make([]int, 0)

	prelude := false

	for _, signal := range signalSequence {
		val := closestValue(signal)
		if val == NEC_INITIATOR || val == NEC_FILLER {
			if len(currentList) > 0 {
				listsOfBits = append(listsOfBits, currentList)
				currentList = make([]int, 0)
				prelude = false
			}

			continue
		}

		if prelude {
			if val == NEC_SHORT {
				currentList = append(currentList, 0)
				prelude = false
			} else if val == NEC_LONG {
				currentList = append(currentList, 1)
				prelude = false
			} else {
				return errors.New(fmt.Sprintf("invalid signal sequence. expected short or long signal, got %v", val))
			}
		} else {
			if val == NEC_SHORT {
				prelude = true
			} else {
				return errors.New(fmt.Sprintf("invalid signal sequence. expected short signal, got %v", val))
			}
		}
	}

	if len(currentList) > 0 {
		listsOfBits = append(listsOfBits, currentList)
	}

	for _, list := range listsOfBits {
		// make sure the list is of the correct length
		if len(list)%8 != 0 {
			return errors.New(fmt.Sprintf("invalid signal sequence. expected length of list to be a multiple of 8, got %v", len(list)))
		}
	}

	if len(listsOfBits) != 2 {
		return errors.New(fmt.Sprintf("invalid signal sequence. expected 2 lists, got %v", len(listsOfBits)))
	}

	if !reflect.DeepEqual(listsOfBits[0], listsOfBits[1]) {
		return errors.New(fmt.Sprintf("invalid signal sequence. expected both lists to be equal, got %v and %v", listsOfBits[0], listsOfBits[1]))
	}

	bytes := make([][]byte, len(listsOfBits))
	for i, list := range listsOfBits {
		bytes[i] = intoBytesLSB(list)
	}

	a := bytes[0][0]
	ar := bytes[0][1]
	b := bytes[0][2]
	br := bytes[0][3]
	c := bytes[0][4]
	cr := bytes[0][5]

	if (a ^ ar) != 0xFF {
		return errors.New(fmt.Sprintf("invalid signal sequence. expected a[0] ^ ar[0] to be 0xFF, got %v", a^ar))
	}

	if (b ^ br) != 0xFF {
		return errors.New(fmt.Sprintf("invalid signal sequence. expected b[0] ^ br[0] to be 0xFF, got %v", b^br))
	}

	if (c ^ cr) != 0xFF {
		return errors.New(fmt.Sprintf("invalid signal sequence. expected c[0] ^ cr[0] to be 0xFF, got %v", c^cr))
	}

	n.cmd[0] = a
	n.cmd[1] = b
	n.cmd[2] = c

	return nil
}

func closestValue(x int) int {
	candidates := []int{NEC_SHORT, NEC_LONG, NEC_INITIATOR, NEC_FILLER}
	closest := candidates[0]
	for _, candidate := range candidates {
		if abs(candidate-x) < abs(closest-x) {
			closest = candidate
		}
	}

	return closest
}

func abs(x int) int {
	if x < 0 {
		return -x
	}
	return x
}

func intoByteLSB(bits []int) byte {

	var result byte = 0
	for i, bit := range bits {
		if bit == 1 {
			result |= 1 << uint(i)
		}
	}

	return result
}

func intoBytesLSB(bits []int) []byte {
	result := make([]byte, 0)

	for i := 0; i < len(bits); i += 8 {
		result = append(result, intoByteLSB(bits[i:i+8]))
	}

	return result
}

func (n *NecChainedCommand) ToSignalSequence() []int {
	return nil
}

func (n *NecChainedCommand) DebugString() string {
	return fmt.Sprintf("NEC Chained Command: %08b %08b %08b", n.cmd[0], n.cmd[1], n.cmd[2])
}
