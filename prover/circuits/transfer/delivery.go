// Delivery v1 adds a proof statement over the ORIGINAL ciphertext recipient wires.
// No encryption, owner authorization, audit, policy or SPP check is replaced.
package transfer

import (
	"github.com/consensys/gnark/frontend"
	"github.com/helius-labs/zolana-custom-rings/prover/circuits/audit"
	"github.com/helius-labs/zolana-custom-rings/prover/circuits/ringgadget"
	"github.com/helius-labs/zolana/prover/server/circuits/gadget"
)

const DeliveryRecipientsDomain = 0x43524b31 // CRK1
const DeliveryStatementDomain = 0x43524431  // CRD1

// DeliveryCommitment binds count, fixed key capacity and each published recipient in order.
// Recipient bytes are bounded again here, independently of the encryption gadget.
// A suffix slot is zero in the chain; no unconstrained unused wire changes the commitment.
func DeliveryCommitment(api frontend.API, outputs []audit.OutputWires, count ringgadget.Count) frontend.Variable {
	fields := []frontend.Variable{DeliveryRecipientsDomain, 0, len(outputs)}
	for i, output := range outputs {
		active := count.Active[i]
		fields[1] = api.Add(fields[1], active)
		fields = append(fields, api.Mul(active, gadget.HashBytes(api, output.RecipientKey[:])))
	}
	fields = append(fields, len(fields)+1)
	return gadget.HashChain4(api, fields)
}

func DeliveryStatement(api frontend.API, original, recipients frontend.Variable) frontend.Variable {
	return gadget.PoseidonHash(api, []frontend.Variable{DeliveryStatementDomain, original, recipients})
}
