package ring

import (
	"encoding/json"
	"os"
	"testing"

	"github.com/helius-labs/zolana-custom-rings/prover/ringtest"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest/ringaudit"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest/ringmerge"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest/ringprotocol"
)

// Rust and TypeScript pin the same file.
const mergeVectorPath = "../../test-vectors/merge/merge.json"

const mergeVectorDescription = "Known-answer vector for the ring merge record and statement. " +
	"record = kind 1, u64be(amount), owner_key, mint, be32(nullifier_pk), be32(output_blinding), be32(seed), 169 bytes. " +
	"envelope = eph_sk || recipient_u, sealed from eph to auditor under the audit stream schedule of label CRING/mrg at counter 2. " +
	"ciphertext seals the record from eph to recipient_u under the CRDS delivered slot secret with a zero salt, index 0 and first_nullifier, label CRING/mro at counter 2. " +
	"output_blinding = Poseidon(TMOB, seed, first_nullifier). " +
	"output_ciphertext[i] = (amount, mint chunk 0, mint chunk 1)[i] + Poseidon(TMOC, Poseidon(TMOK, seed, first_nullifier), output_ciphertext_salt, i). " +
	"elements = CRMG, output_utxo_hash, first_nullifier, ring_program_id, auditor lo, hi, eph lo, hi, hash_bytes(envelope_ciphertext), hash_bytes(ciphertext), payer, tracking_root, output_ciphertext_salt, output_ciphertext, " +
	"public_input_hash = hash_chain(elements). seed is the owner's nullifier secret. " +
	"request is the prover body. Produced by prover/ring/merge_test.go with WRITE_AUDIT_VECTORS=1."

type mergeVectorCase struct {
	Name                 string       `json:"name"`
	Record               Bytes        `json:"record"`
	Envelope             Bytes        `json:"envelope"`
	EnvelopeCiphertext   Bytes        `json:"envelope_ciphertext"`
	Ciphertext           Bytes        `json:"ciphertext"`
	OutputBlinding       Field        `json:"output_blinding"`
	OutputCiphertextSalt Field        `json:"output_ciphertext_salt"`
	OutputCiphertext     []Field      `json:"output_ciphertext"`
	Elements             []Field      `json:"elements"`
	PublicInputHash      Field        `json:"public_input_hash"`
	Request              *MergeParams `json:"request"`
}

type mergeVector struct {
	Description string            `json:"description"`
	AuditorSk   Bytes             `json:"auditor_sk"`
	AuditorPk   Bytes             `json:"auditor_pk"`
	EphPk       Bytes             `json:"eph_pk"`
	Cases       []mergeVectorCase `json:"cases"`
}

func mergeCase(t *testing.T, name string, s *ringmerge.Statement) mergeVectorCase {
	t.Helper()
	return mergeVectorCase{
		Name:                 name,
		Record:               s.Record,
		Envelope:             s.PlainEnvelope(),
		EnvelopeCiphertext:   s.EnvelopeCiphertext(t),
		Ciphertext:           s.Ciphertext(t),
		OutputBlinding:       Field{s.OutputBlinding},
		OutputCiphertextSalt: Field{ringtest.AsBigInt(s.Circuit.OutputCiphertextSalt)},
		OutputCiphertext:     fieldsOf(s.OutputCiphertext),
		Elements:             fieldsOf(s.Elements(t)),
		PublicInputHash:      Field{ringtest.AsBigInt(s.Circuit.PublicInputHash)},
		Request:              requestOf[MergeParams](t, mergeKey, s.Circuit),
	}
}

func TestMergeVectorPinsHostAndCircuit(t *testing.T) {
	s := ringmerge.Build(t)
	auditorSk := ringaudit.AuditorSk()
	auditorU := s.Keys.AuditorPk()
	auditorPk := ringprotocol.Tagged(auditorU[:])
	ephPk := s.Keys.EphPk()
	want := mergeVector{
		Description: mergeVectorDescription,
		AuditorSk:   auditorSk[:],
		AuditorPk:   auditorPk[:],
		EphPk:       ephPk[:],
		Cases:       []mergeVectorCase{mergeCase(t, "enrolled", s)},
	}
	encoded, err := json.MarshalIndent(want, "", "  ")
	if err != nil {
		t.Fatal(err)
	}
	encoded = append(encoded, '\n')
	if os.Getenv("WRITE_AUDIT_VECTORS") != "" {
		if err := os.WriteFile(mergeVectorPath, encoded, 0o644); err != nil {
			t.Fatal(err)
		}
	}
	raw, err := os.ReadFile(mergeVectorPath)
	if err != nil {
		t.Fatal(err)
	}
	if string(raw) != string(encoded) {
		t.Fatal("the host mirror disagrees with test-vectors/merge/merge.json, rewrite it with WRITE_AUDIT_VECTORS=1")
	}
	var pinned mergeVector
	if err := json.Unmarshal(raw, &pinned); err != nil {
		t.Fatal(err)
	}
	for _, vector := range pinned.Cases {
		if _, err := solvedWitness(t, mergeKey, mustMarshal(t, vector.Request)); err != nil {
			t.Fatal(vector.Name, err)
		}
	}
}
