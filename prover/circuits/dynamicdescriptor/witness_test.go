package descriptor

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"strings"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/test"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest"
)

func fixtureRequest(t *testing.T) WitnessRequest {
	c := fixture(t)
	f := func(v frontend.Variable) string { return ringtest.AsBigInt(v).String() }
	h := func(v []frontend.Variable) string {
		b := make([]byte, len(v))
		for i, v := range v {
			b[i] = byte(ringtest.AsBigInt(v).Uint64())
		}
		return hex.EncodeToString(b)
	}
	r := WitnessRequest{Version: 1, StatementHash: f(c.PublicInputHash), RingID: f(c.RingID), TrackingRoot: f(c.TrackingRoot), RingGeneration: f(c.RingGeneration), OperationContext: f(c.OperationContext), OutputIndex: 1, SourceAuthorityIdentity: f(c.SourceAuthorityIdentity), PayerIdentity: f(c.PayerIdentity), TreeID: f(c.TreeID), Asset: f(c.Asset), OwnerUtxoHash: f(c.OwnerUtxoHash), RingDataHash: f(c.RingDataHash), MinAmount: f(c.MinAmount), MaxAmount: f(c.MaxAmount), ExpirySlot: f(c.ExpirySlot), RecipientTag: f(c.RecipientTag), AuditorPubkeyHex: "04" + h(c.AuditorPk[:]), TxViewingPubkeyHex: "04" + h(c.TxViewingPk[:]), SaltHex: h(c.Salt[:]), RecipientZeroCiphertextHex: h(c.RecipientCiphertext[:]), AuditorCiphertextHex: h(c.AuditorCiphertext[:])}
	p := &r.Private
	p.OwnerKeyHex = h(c.OwnerKey[:])
	p.NullifierPk = f(c.NullifierPk)
	p.ViewingSeq = f(c.ViewingSeq)
	p.Blinding = f(c.Blinding)
	p.RecipientViewingUHex = h(c.RecipientPk[:])
	p.TxViewingScalarHex = h(c.TxViewingSk[:])
	// Record's mint is synthetic fixture 0x75 repeated 32; no real wallet data.
	p.MintHex = strings.Repeat("75", 32)
	p.Enrollment.Present = true
	record := &p.Enrollment.Record
	source := c.Enrollment.Record
	record.Member = f(source.Member)
	record.Next = f(source.Next)
	record.NullifierPk = f(source.NullifierPk)
	record.CtHash = f(source.CtHash)
	record.ListTag = f(source.ListTag)
	record.ViewingHash = f(source.ViewingHash)
	p.Enrollment.Path.Index = f(c.Enrollment.Path.Index)
	p.Enrollment.Path.Siblings = make([][]string, 16)
	for i, level := range c.Enrollment.Path.Siblings {
		p.Enrollment.Path.Siblings[i] = make([]string, 3)
		for j, v := range level {
			p.Enrollment.Path.Siblings[i][j] = f(v)
		}
	}
	return r
}

func TestWitnessRequestStrictAndLossless(t *testing.T) {
	request := fixtureRequest(t)
	encoded, err := json.Marshal(request)
	if err != nil {
		t.Fatal(err)
	}
	assignment, err := ReadWitnessRequest(bytes.NewReader(encoded))
	if err != nil {
		t.Fatal(err)
	}
	if fmt.Sprint(assignment.PublicInputHash) != request.StatementHash {
		t.Fatal("field lost precision")
	}
	if err := test.IsSolved(&Circuit{}, assignment, ecc.BN254.ScalarField()); err != nil {
		t.Fatal(err)
	}
	for name, edit := range map[string]func(*WitnessRequest){
		"noncanonical field": func(r *WitnessRequest) { r.OperationContext = "0991" },
		"overflow amount":    func(r *WitnessRequest) { r.MaxAmount = "18446744073709551616" },
		"negative amount":    func(r *WitnessRequest) { r.MinAmount = "-1" },
		"wrong path height": func(r *WitnessRequest) {
			r.Private.Enrollment.Path.Siblings = append(r.Private.Enrollment.Path.Siblings, []string{"0", "0", "0"})
		},
		"wrong path width":  func(r *WitnessRequest) { r.Private.Enrollment.Path.Siblings[0] = []string{"0", "0", "0", "0"} },
		"absent enrollment": func(r *WitnessRequest) { r.Private.Enrollment.Present = false },
		"wrong key tag":     func(r *WitnessRequest) { r.AuditorPubkeyHex = "05" + r.AuditorPubkeyHex[2:] },
		"wrong byte length": func(r *WitnessRequest) { r.Private.OwnerKeyHex = "1234" },
	} {
		t.Run(name, func(t *testing.T) {
			r := fixtureRequest(t)
			edit(&r)
			encoded, _ := json.Marshal(r)
			if _, err := ReadWitnessRequest(bytes.NewReader(encoded)); err == nil {
				t.Fatal("malformed witness accepted")
			}
		})
	}
	numeric := bytes.Replace(encoded, []byte(`"operation_context":"991"`), []byte(`"operation_context":991`), 1)
	if _, err := ReadWitnessRequest(bytes.NewReader(numeric)); err == nil {
		t.Fatal("numeric field accepted")
	}
	unknown := append([]byte(`{"sensitive_unknown":"do-not-echo",`), encoded[1:]...)
	if _, err := ReadWitnessRequest(bytes.NewReader(unknown)); err == nil || strings.Contains(err.Error(), "do-not-echo") {
		t.Fatal("unknown field accepted or secret echoed")
	}
	if _, err := ReadWitnessRequest(bytes.NewReader(append(encoded, []byte(` {}`)...))); err == nil {
		t.Fatal("trailing JSON accepted")
	}
}
