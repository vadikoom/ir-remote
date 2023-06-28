package commands

import (
	"github.com/stretchr/testify/require"
	"testing"
)

func TestSimple(t *testing.T) {
	cmd := NecChainedCommand{}
	err := cmd.ParseFromSignalSequence(commandOff)
	require.NoError(t, err)
	println(cmd.DebugString())

	err = cmd.ParseFromSignalSequence(commandCold20)
	require.NoError(t, err)
	println(cmd.DebugString())

	err = cmd.ParseFromSignalSequence(commandCold22)
	require.NoError(t, err)
	println(cmd.DebugString())

	err = cmd.ParseFromSignalSequence(commandCold24)
	require.NoError(t, err)
	println(cmd.DebugString())

	err = cmd.ParseFromSignalSequence(commandWater20)
	require.NoError(t, err)
	println(cmd.DebugString())

	err = cmd.ParseFromSignalSequence(commandWater23)
	require.NoError(t, err)
	println(cmd.DebugString())

	err = cmd.ParseFromSignalSequence(commandWater24)
	require.NoError(t, err)
	println(cmd.DebugString())
}
