package irremote

type Command struct {
	Data           []int `json:"data"`
	SequenceNumber int64 `json:"sequence"`
}

type Status struct {
	LastCommandSequenceNumber int64 `json:"last_command_sequence_number"`
}
