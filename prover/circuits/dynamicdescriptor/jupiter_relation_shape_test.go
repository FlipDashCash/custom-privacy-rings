package descriptor

import (
	"reflect"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
)

// Inspect the exact owned relation as an in-memory symbolic circuit only.
// The compile parameters and shape checks match the first part of
// TestDescriptorNativeConstraintSystem in owned Custom #5, f841ff6.
// This target has no witness fixture, proof or key use, verifier invocation,
// output artifact, endpoint, or runtime registration.
func TestJupiterDescriptorRelationConstraints(t *testing.T) {
	symbol, ok := reflect.TypeOf(Circuit{}).FieldByName("PublicInputHash")
	if !ok || symbol.Tag.Get("gnark") != ",public" {
		t.Fatal("descriptor's public symbol must remain PublicInputHash")
	}
	cs, err := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &Circuit{}, frontend.WithCompressThreshold(300))
	if err != nil {
		t.Fatal(err)
	}
	commitments, ok := cs.GetCommitments().(constraint.Groth16Commitments)
	if !ok || len(commitments) != 1 || commitments[0].NbPublicCommitted != 0 {
		t.Fatal("descriptor must retain exactly one entirely private BSB22 commitment")
	}
	publicInputs := cs.GetNbPublicVariables() - 1 // exclude the R1CS constant-one wire
	if publicInputs != 1 {
		t.Fatalf("expected one public statement hash, got %d", publicInputs)
	}
	constraints := cs.GetNbConstraints()
	if constraints <= 0 {
		t.Fatal("descriptor has no compiled constraints")
	}
	t.Logf("relation-only: curve=BN254 compression_threshold=300 constraints=%d public_inputs=%d public_input=PublicInputHash private_bsb22_commitments=%d public_committed=%d",
		constraints, publicInputs, len(commitments), commitments[0].NbPublicCommitted)
}
