package transfer_test

import (
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/test"
	"github.com/helius-labs/zolana-custom-rings/prover/circuits/transfer"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest/ringaudit"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest/ringtransfer"
	"testing"
)

// Actual complete original ciphertext/policy relation with the additional recipient relation.
func TestDeliveryV1FullRelation(t *testing.T) {
	for _, shape := range []transfer.Shape{{Inputs: 24, Outputs: 4, Withdrawals: 4, Owners: 4}, {Inputs: 16, Outputs: 8, Withdrawals: 4, Owners: 4}} {
		t.Run(shape.String(), func(t *testing.T) {
			w := ringtransfer.Hidden(shape)
			honest := w.Build(t)
			original := ringtest.AsBigInt(honest.Circuit.PublicInputHash)
			expected := ringtransfer.BindDeliveryV1(t, honest.Circuit)
			requireSolved(t, honest.Circuit)
			if ringtest.AsBigInt(honest.Circuit.PublicInputHash).Cmp(original) == 0 {
				t.Fatal("old/new domain collision")
			}
			old := w.Build(t)
			requireSolved(t, old.Circuit) // unchanged v15 still solves.
			if test.IsSolved(old.Circuit, honest.Circuit, ecc.BN254.ScalarField()) == nil {
				t.Fatal("new statement accepted by classic relation")
			}
			// Valid owner and note, ciphertext honestly sealed to a different recipient key.
			// All original audit constraints still hold; the immutable expected recipient fails.
			changed := w
			changed.Outputs = map[int]func(*ringaudit.Slot){0: func(s *ringaudit.Slot) { s.RecipientKey = ringaudit.ViewingKey(t, 0x42) }}
			wrong := changed.Build(t)
			requireSolved(t, changed.Build(t).Circuit)
			if ringtransfer.DeliveryCommitment(t, wrong.Circuit).Cmp(expected) == 0 {
				t.Fatal("wrong viewing key aliases")
			}
			ringtransfer.BindDeliveryTo(t, wrong.Circuit, expected)
			if test.IsSolved(wrong.Circuit, wrong.Circuit, ecc.BN254.ScalarField()) == nil {
				t.Fatal("correct owner with wrong viewing key solved immutable delivery")
			}
			// Same new key cannot accept the unchanged classic public input.
			same := w.Build(t)
			same.Circuit.DeliveryBound = true
			if test.IsSolved(same.Circuit, same.Circuit, ecc.BN254.ScalarField()) == nil {
				t.Fatal("classic public input solved delivery relation")
			}
		})
	}
}

func TestDeliveryV1CountOrderCapacityVector(t *testing.T) {
	shape := transfer.Shape{Inputs: 24, Outputs: 4, Withdrawals: 4, Owners: 4}
	w := ringtransfer.Signed(shape)
	c := w.Build(t).Circuit
	// A small known-answer fixture consumed by Rust as a hex constant.
	for i := range c.Outputs {
		for j := range c.Outputs[i].RecipientKey {
			c.Outputs[i].RecipientKey[j] = i + 1
		}
	}
	c.OutputCount = 2
	expected := ringtransfer.DeliveryCommitment(t, c)
	if expected.Sign() == 0 {
		t.Fatal("zero commitment")
	}
	t.Logf("delivery_vector=%064x", expected)
	c.Outputs[0].RecipientKey, c.Outputs[1].RecipientKey = c.Outputs[1].RecipientKey, c.Outputs[0].RecipientKey
	if ringtransfer.DeliveryCommitment(t, c).Cmp(expected) == 0 {
		t.Fatal("order not bound")
	}
	c.OutputCount = 1
	if ringtransfer.DeliveryCommitment(t, c).Cmp(expected) == 0 {
		t.Fatal("count not bound")
	}
	if _, err := transfer.NewDeliveryTransferCircuit(transfer.Shape{Inputs: 24, Outputs: 2, Withdrawals: 4, Owners: 4}); err == nil {
		t.Fatal("unkeyed width accepted")
	}
}
