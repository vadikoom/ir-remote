package encoder

import "encoding/json"

type dummyEncoder struct {
}

func NewDummyEncoder() Encoder {
	return &dummyEncoder{}
}

func (d *dummyEncoder) Encrypt(message any) []byte {
	out, err := json.Marshal(message)
	if err != nil {
		panic(err)
	}
	return out
}

func (d *dummyEncoder) Decrypt(data []byte, into any) error {
	return json.Unmarshal(data, into)
}
