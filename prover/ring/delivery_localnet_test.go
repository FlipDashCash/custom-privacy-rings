//go:build pyrol_delivery_localnet

package ring

import (
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/test"
	"testing"
)

func TestDeliveryLocalRequestsDecodeAndSolve(t *testing.T) {
	for _, k := range Keys {
		if k.Family != DeliveryTransfer {
			continue
		}
		t.Run(string(k.Circuit), func(t *testing.T) {
			payload := mustMarshal(t, hostRequest(t, k))
			circuit := extensionOf(t, k.Circuit).Circuit()
			if err := test.IsSolved(circuit, decodePayload(t, k.Circuit, payload), ecc.BN254.ScalarField()); err != nil {
				t.Fatal(err)
			}
		})
	}
}
