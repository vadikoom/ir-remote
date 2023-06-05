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
	println("Encrypting ", string(out))
	return out
}

func (d *dummyEncoder) Decrypt(data []byte, into any) error {
	err := json.Unmarshal(data, into)
	if err != nil {
	    return err
    }

    println("Decrypted ", string(data))
    return nil
}
