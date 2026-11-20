// Test-only host mirror of the delivery-v1 statement.
package ringtransfer

import (
	"github.com/helius-labs/zolana-custom-rings/prover/circuits/transfer"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest"
	"github.com/helius-labs/zolana/prover/server/prover-test/spp/spptest"
	"math/big"
	"testing"
)

func DeliveryCommitment(t testing.TB, c *transfer.TransferCircuit) *big.Int {
	t.Helper()
	count := int(ringtest.AsBigInt(c.OutputCount).Int64())
	fields := []*big.Int{big.NewInt(transfer.DeliveryRecipientsDomain), big.NewInt(int64(count)), big.NewInt(int64(len(c.Outputs)))}
	for i, output := range c.Outputs {
		key := make([]byte, 32)
		for j, b := range output.RecipientKey {
			key[j] = byte(ringtest.AsBigInt(b).Uint64())
		}
		value := big.NewInt(0)
		if i < count {
			value = ringtest.HashBytes(t, key)
		}
		fields = append(fields, value)
	}
	fields = append(fields, big.NewInt(int64(len(fields)+1)))
	return spptest.MustHashChain4(t, fields)
}

func BindDeliveryV1(t testing.TB, c *transfer.TransferCircuit) *big.Int {
	t.Helper()
	commitment := DeliveryCommitment(t, c)
	BindDeliveryTo(t, c, commitment)
	return commitment
}

func BindDeliveryTo(t testing.TB, c *transfer.TransferCircuit, commitment *big.Int) {
	t.Helper()
	c.DeliveryBound = true
	c.PublicInputHash = ringtest.MustPoseidonHash(t, big.NewInt(transfer.DeliveryStatementDomain), ringtest.AsBigInt(c.PublicInputHash), commitment)
}
