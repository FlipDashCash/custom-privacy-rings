package descriptor

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"math/big"
	"os"
	"strconv"
	"testing"

	"github.com/helius-labs/zolana/prover/server/prover-test/spp/protocol"
)

func parityHex(t *testing.T, row map[string]string, field string, width int) []byte {
	t.Helper()
	raw, err := hex.DecodeString(row[field])
	if err != nil || len(raw) != width {
		t.Fatalf("wrong synthetic public hex width: %s", field)
	}
	return raw
}

func parityDecimal(t *testing.T, row map[string]string, field string, limit int) *big.Int {
	t.Helper()
	word, err := strconv.ParseUint(row[field], 10, limit)
	if err != nil {
		t.Fatalf("wrong synthetic decimal width: %s", field)
	}
	return new(big.Int).SetUint64(word)
}

func parityHashBytes(t *testing.T, raw []byte) *big.Int {
	t.Helper()
	word, err := protocol.HashBytes(raw)
	if err != nil {
		t.Fatal(err)
	}
	return word
}

func parityRoleIdentity(t *testing.T, raw []byte) *big.Int {
	t.Helper()
	word, err := protocol.SolanaPkField([32]byte(raw))
	if err != nil {
		t.Fatal(err)
	}
	return word
}

func parityPack33(t *testing.T, raw []byte) [2]*big.Int {
	t.Helper()
	if len(raw) != 33 || raw[0] != 4 {
		t.Fatal("expected a synthetic tagged public word")
	}
	// This is only the public 31+2 byte encoding of shared.PackedKey,
	// not a curve operation or an alternative hashing implementation.
	return [2]*big.Int{new(big.Int).SetBytes(raw[:31]), new(big.Int).SetBytes(raw[31:])}
}

// Use the same named public fixture as the Rust frame. Use only pinned native
// Go protocol helpers, with the field order of the exact descriptor relation.
// No witness input, key material, circuit solver, or verification is involved.
func TestJupiterDescriptorPublicInputParity(t *testing.T) {
	data, err := os.ReadFile("../../../test-vectors/settlement/jupiter_measured_descriptor_v1_public_input.json")
	if err != nil {
		t.Fatal(err)
	}
	var row map[string]string
	if err := json.Unmarshal(data, &row); err != nil {
		t.Fatal(err)
	}
	if row["schema"] != "pyrol-custom-jupiter-v1-public-statement-parity" {
		t.Fatal("wrong public fixture domain")
	}
	hexField := func(field string, width int) []byte { return parityHex(t, row, field, width) }
	scalar := func(field string, width int) *big.Int { return new(big.Int).SetBytes(hexField(field, width)) }
	hashed := func(field string, width int) *big.Int { return parityHashBytes(t, hexField(field, width)) }
	auditor := parityPack33(t, hexField("auditorPublicWord", 33))
	viewing := parityPack33(t, hexField("txViewingPublicWord", 33))
	asset, err := protocol.AssetField([32]byte(hexField("outputMint", 32)))
	if err != nil {
		t.Fatal(err)
	}
	cells := []*big.Int{
		big.NewInt(Domain),
		big.NewInt(Version),
		hashed("ringProgram", 32),
		auditor[0], auditor[1],
		scalar("trackingRoot", 32),
		parityDecimal(t, row, "ringGeneration", 32),
		hashed("checkedContextPreimageSha256", 32),
		new(big.Int), // single-output ordinal
		parityRoleIdentity(t, hexField("sourceAuthority", 32)),
		parityRoleIdentity(t, hexField("feePayer", 32)),
		scalar("treeId", 32),
		asset,
		scalar("ownerCommitment", 32),
		new(big.Int), // frozen descriptor has no ring data
		parityDecimal(t, row, "minimumOutputAtoms", 64),
		new(big.Int).SetUint64(^uint64(0)),
		parityDecimal(t, row, "expirySlot", 64),
		viewing[0], viewing[1],
		scalar("salt", 16),
		scalar("recipientTag", 32),
		hashed("recipientCiphertext", RecipientLen),
		hashed("auditorCiphertext", AuditorLen),
	}
	cells = append(cells, new(big.Int).SetUint64(uint64(len(cells)+1)))
	if len(cells) != 25 {
		t.Fatal("wrong CRDD preimage width")
	}
	hash, err := protocol.HashChain4(cells)
	if err != nil {
		t.Fatal(err)
	}
	var wire bytes.Buffer
	for _, cell := range cells {
		wire.Write(cell.FillBytes(make([]byte, 32)))
	}
	fmt.Printf("PYROL_JUPITER_V1_PUBLIC_PARITY %x.%x\n", wire.Bytes(), hash.FillBytes(make([]byte, 32)))
}
