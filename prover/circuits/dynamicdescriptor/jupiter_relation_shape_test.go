package descriptor

import (
	"crypto/sha256"
	"encoding/hex"
	"hash"
	"reflect"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
)

// Inspect the exact owned relation as an in-memory symbolic circuit only.
// The compile parameters, shape checks, and WriteTo serializer are the same
// as TestDescriptorNativeConstraintSystem in owned Custom #5, f841ff6.
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

	// The keyless Custom #5 qualification recorded this hash of cs.WriteTo.
	const wantR1CSSHA256 = "746aea82a384ca80d53c988242371a1789a04d4dc88b4d5390aa7a40dfaaeaa1"
	digest := &countedR1CSHash{Hash: sha256.New()}
	n, err := cs.WriteTo(digest)
	if err != nil {
		t.Fatalf("serialize descriptor R1CS: %v", err)
	}
	if n <= 0 || n != digest.bytes {
		t.Fatalf("descriptor R1CS byte count: WriteTo reported %d, SHA-256 received %d", n, digest.bytes)
	}
	got := hex.EncodeToString(digest.Sum(nil))
	if got != wantR1CSSHA256 {
		t.Fatalf("descriptor R1CS SHA-256 mismatch (%d bytes): got %s, want %s", n, got, wantR1CSSHA256)
	}
	t.Logf("NO_SETUP NO_PROOF: r1cs_bytes=%d r1cs_sha256=%s", n, got)
}

type countedR1CSHash struct {
	hash.Hash
	bytes int64
}

func (w *countedR1CSHash) Write(p []byte) (int, error) {
	n, err := w.Hash.Write(p)
	w.bytes += int64(n)
	return n, err
}
