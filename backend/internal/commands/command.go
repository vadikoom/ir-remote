package commands

type Command interface {
	ParseFromSignalSequence(signalSequence []int) error
	ToSignalSequence() []int
}
