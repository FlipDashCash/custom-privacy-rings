package descriptor

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"math/big"
	"strings"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/frontend"
	"github.com/helius-labs/zolana-custom-rings/prover/circuits/registry"
)

// WitnessRequest is private prover input. Never log, cache or return this object.
// Scalars/amounts use canonical decimal strings; raw bytes are exact-length hex.
// It intentionally excludes wallet signatures and spend/nullifier secret keys.
type WitnessRequest struct {
	Version                    int            `json:"version"`
	StatementHash              string         `json:"statement_hash"`
	RingID                     string         `json:"ring_id"`
	TrackingRoot               string         `json:"tracking_root"`
	RingGeneration             string         `json:"ring_generation"`
	OperationContext           string         `json:"operation_context"`
	OutputIndex                uint8          `json:"output_index"`
	SourceAuthorityIdentity    string         `json:"source_authority_identity"`
	PayerIdentity              string         `json:"payer_identity"`
	TreeID                     string         `json:"tree_id"`
	Asset                      string         `json:"asset"`
	OwnerUtxoHash              string         `json:"owner_utxo_hash"`
	RingDataHash               string         `json:"ring_data_hash"`
	MinAmount                  string         `json:"min_amount"`
	MaxAmount                  string         `json:"max_amount"`
	ExpirySlot                 string         `json:"expiry_slot"`
	RecipientTag               string         `json:"recipient_tag"`
	AuditorPubkeyHex           string         `json:"auditor_pubkey_hex"`
	TxViewingPubkeyHex         string         `json:"tx_viewing_pubkey_hex"`
	SaltHex                    string         `json:"salt_hex"`
	RecipientZeroCiphertextHex string         `json:"recipient_zero_ciphertext_hex"`
	AuditorCiphertextHex       string         `json:"auditor_ciphertext_hex"`
	Private                    PrivateWitness `json:"private"`
}

type PrivateWitness struct {
	ViewingSeq           string            `json:"viewing_seq"`
	OwnerKeyHex          string            `json:"owner_key_hex"`
	NullifierPk          string            `json:"nullifier_pk"`
	Blinding             string            `json:"blinding"`
	RecipientViewingUHex string            `json:"recipient_viewing_u_hex"`
	TxViewingScalarHex   string            `json:"tx_viewing_scalar_hex"`
	MintHex              string            `json:"mint_hex"`
	Enrollment           EnrollmentWitness `json:"enrollment"`
}

type EnrollmentWitness struct {
	Present bool `json:"present"`
	Record  struct {
		Member      string `json:"member"`
		Next        string `json:"next"`
		NullifierPk string `json:"nullifier_pk"`
		CtHash      string `json:"ct_hash"`
		ListTag     string `json:"list_tag"`
		ViewingHash string `json:"viewing_hash"`
	} `json:"record"`
	Path struct {
		Index    string     `json:"index"`
		Siblings [][]string `json:"siblings"`
	} `json:"path"`
}

// ReadWitnessRequest strictly decodes one bounded request. Parse errors never
// echo secret values, unlike raw encoding/json type errors.
func ReadWitnessRequest(r io.Reader) (*Circuit, error) {
	data, err := io.ReadAll(io.LimitReader(r, 128*1024+1))
	if err != nil || len(data) > 128*1024 {
		return nil, errors.New("descriptor request exceeds allowed size")
	}
	decoder := json.NewDecoder(bytes.NewReader(data))
	decoder.DisallowUnknownFields()
	var request WitnessRequest
	if err := decoder.Decode(&request); err != nil {
		return nil, errors.New("invalid descriptor request JSON")
	}
	var trailing interface{}
	if err := decoder.Decode(&trailing); err != io.EOF {
		return nil, errors.New("trailing or oversized descriptor request")
	}
	return request.Circuit()
}

func decimalField(value, path string, bits int) (*big.Int, error) {
	if value == "" || (len(value) > 1 && value[0] == '0') {
		return nil, fmt.Errorf("%s must be a canonical decimal string", path)
	}
	for _, c := range value {
		if c < '0' || c > '9' {
			return nil, fmt.Errorf("%s must be a canonical decimal string", path)
		}
	}
	v, ok := new(big.Int).SetString(value, 10)
	if !ok || v.Cmp(ecc.BN254.ScalarField()) >= 0 || (bits > 0 && v.BitLen() > bits) {
		return nil, fmt.Errorf("%s is outside its allowed range", path)
	}
	return v, nil
}

func fixedHex(value, path string, n int) ([]byte, error) {
	if len(value) != n*2 || value != strings.ToLower(value) {
		return nil, fmt.Errorf("%s must be exact-length lowercase hex", path)
	}
	bytes, err := hex.DecodeString(value)
	if err != nil {
		return nil, fmt.Errorf("%s is invalid hex", path)
	}
	return bytes, nil
}

func (r WitnessRequest) Circuit() (*Circuit, error) {
	if r.Version != Version {
		return nil, errors.New("unsupported descriptor version")
	}
	if r.OutputIndex > 4 {
		return nil, errors.New("output_index is outside 0..4")
	}
	c := &Circuit{OutputIndex: r.OutputIndex}
	assign := func(dst *frontend.Variable, value, path string, bits int) error {
		v, err := decimalField(value, path, bits)
		if err == nil {
			*dst = v
		}
		return err
	}
	fields := []struct {
		dst         *frontend.Variable
		value, path string
		bits        int
	}{
		{&c.PublicInputHash, r.StatementHash, "statement_hash", 0}, {&c.RingID, r.RingID, "ring_id", 0},
		{&c.TrackingRoot, r.TrackingRoot, "tracking_root", 0}, {&c.RingGeneration, r.RingGeneration, "ring_generation", 32},
		{&c.OperationContext, r.OperationContext, "operation_context", 0}, {&c.SourceAuthorityIdentity, r.SourceAuthorityIdentity, "source_authority_identity", 0},
		{&c.PayerIdentity, r.PayerIdentity, "payer_identity", 0}, {&c.TreeID, r.TreeID, "tree_id", 16},
		{&c.Asset, r.Asset, "asset", 0}, {&c.OwnerUtxoHash, r.OwnerUtxoHash, "owner_utxo_hash", 0},
		{&c.RingDataHash, r.RingDataHash, "ring_data_hash", 0}, {&c.MinAmount, r.MinAmount, "min_amount", 64},
		{&c.MaxAmount, r.MaxAmount, "max_amount", 64}, {&c.ExpirySlot, r.ExpirySlot, "expiry_slot", 64},
		{&c.RecipientTag, r.RecipientTag, "recipient_tag", 64}, {&c.NullifierPk, r.Private.NullifierPk, "private.nullifier_pk", 0},
		{&c.Blinding, r.Private.Blinding, "private.blinding", 0},
		{&c.ViewingSeq, r.Private.ViewingSeq, "private.viewing_seq", 64},
	}
	for _, field := range fields {
		if err := assign(field.dst, field.value, field.path, field.bits); err != nil {
			return nil, err
		}
	}
	bytesInto := func(dst []frontend.Variable, value, path string) error {
		b, err := fixedHex(value, path, len(dst))
		if err != nil {
			return err
		}
		for i, v := range b {
			dst[i] = v
		}
		return nil
	}
	taggedInto := func(dst []frontend.Variable, value, path string) error {
		b, err := fixedHex(value, path, 33)
		if err != nil {
			return err
		}
		if b[0] != 4 {
			return fmt.Errorf("%s requires official X25519 tag", path)
		}
		for i, v := range b[1:] {
			dst[i] = v
		}
		return nil
	}
	if err := taggedInto(c.AuditorPk[:], r.AuditorPubkeyHex, "auditor_pubkey_hex"); err != nil {
		return nil, err
	}
	if err := taggedInto(c.TxViewingPk[:], r.TxViewingPubkeyHex, "tx_viewing_pubkey_hex"); err != nil {
		return nil, err
	}
	bytefields := []struct {
		dst         []frontend.Variable
		value, path string
	}{
		{c.Salt[:], r.SaltHex, "salt_hex"}, {c.RecipientCiphertext[:], r.RecipientZeroCiphertextHex, "recipient_zero_ciphertext_hex"},
		{c.AuditorCiphertext[:], r.AuditorCiphertextHex, "auditor_ciphertext_hex"}, {c.OwnerKey[:], r.Private.OwnerKeyHex, "private.owner_key_hex"},
		{c.RecipientPk[:], r.Private.RecipientViewingUHex, "private.recipient_viewing_u_hex"}, {c.TxViewingSk[:], r.Private.TxViewingScalarHex, "private.tx_viewing_scalar_hex"},
	}
	for _, f := range bytefields {
		if err := bytesInto(f.dst, f.value, f.path); err != nil {
			return nil, err
		}
	}
	mint, err := fixedHex(r.Private.MintHex, "private.mint_hex", 32)
	if err != nil {
		return nil, err
	}
	c.MintChunks[0], c.MintChunks[1] = new(big.Int).SetBytes(mint[:31]), new(big.Int).SetUint64(uint64(mint[31]))
	if !r.Private.Enrollment.Present {
		return nil, errors.New("private.enrollment must be present")
	}
	c.Enrollment.Present = 1
	record := &c.Enrollment.Record
	input := r.Private.Enrollment.Record
	records := []struct {
		dst         *frontend.Variable
		value, path string
	}{
		{&record.Member, input.Member, "member"}, {&record.Next, input.Next, "next"}, {&record.NullifierPk, input.NullifierPk, "nullifier_pk"},
		{&record.CtHash, input.CtHash, "ct_hash"}, {&record.ListTag, input.ListTag, "list_tag"}, {&record.ViewingHash, input.ViewingHash, "viewing_hash"},
	}
	for _, f := range records {
		if err := assign(f.dst, f.value, "private.enrollment.record."+f.path, 0); err != nil {
			return nil, err
		}
	}
	if err := assign(&c.Enrollment.Path.Index, r.Private.Enrollment.Path.Index, "private.enrollment.path.index", registry.TrackingIndexBits); err != nil {
		return nil, err
	}
	if len(r.Private.Enrollment.Path.Siblings) != registry.TrackingHeight {
		return nil, errors.New("private.enrollment path requires exactly 16 levels")
	}
	for level, siblings := range r.Private.Enrollment.Path.Siblings {
		if len(siblings) != registry.Arity-1 {
			return nil, errors.New("private.enrollment path requires exactly 3 siblings per level")
		}
		for index, v := range siblings {
			if err := assign(&c.Enrollment.Path.Siblings[level][index], v, "private.enrollment.path.siblings", 0); err != nil {
				return nil, err
			}
		}
	}
	return c, nil
}
