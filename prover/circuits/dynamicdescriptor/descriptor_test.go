package descriptor

import (
	"crypto/ecdh"
	"crypto/sha256"
	"encoding/binary"
	"math/big"
	"slices"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/consensys/gnark/test"
	"github.com/helius-labs/zolana-custom-rings/prover/circuits/shared"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest/ringprotocol"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest/ringregistry"
	"github.com/helius-labs/zolana/prover/server/prover-test/spp/protocol"
	"github.com/helius-labs/zolana/prover/server/prover-test/spp/spptest"
)

func fixture(t *testing.T) *Circuit { return fixtureAt(t, 1) }

func fixtureAt(t *testing.T, index int) *Circuit {
	t.Helper()
	c := &Circuit{RingID: 37, RingGeneration: 2, OperationContext: 991, OutputIndex: index,
		SourceAuthorityIdentity: 51, PayerIdentity: 52, TreeID: 7, RingDataHash: 0,
		MinAmount: 2, MaxAmount: 10_000, ExpirySlot: 5000}
	ownerKey := ringtest.Repeat(0x71)
	owner := ringtest.MustSolanaPkField(t, ownerKey)
	recipientSk := ringprotocol.Clamped(ringtest.Scalar(0x72))
	recipientPk := ringprotocol.XPublicKey(t, recipientSk)
	txSk := ringprotocol.Clamped(ringtest.Scalar(0x73))
	txPk := ringprotocol.XPublicKey(t, txSk)
	auditorPk := ringprotocol.XPublicKey(t, ringprotocol.Clamped(ringtest.Scalar(0x74)))
	nullifierPk := ringtest.MustPoseidonHash(t, big.NewInt(1234567))
	mint := ringtest.Repeat(0x75)
	asset := ringtest.AssetField(t, mint)
	salt := [16]byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16}
	context := big.NewInt(991)
	seed := ringprotocol.BlindingSeed(t, txSk, salt)
	outputSeed, err := protocol.OutputBlindingSeed(context, seed)
	if err != nil {
		t.Fatal(err)
	}
	blinding := ringprotocol.OutputBlinding(t, context, outputSeed, index)
	c.Blinding, c.NullifierPk, c.Asset, c.ViewingSeq = blinding, nullifierPk, asset, 0
	c.OwnerUtxoHash = ringtest.MustPoseidonHash(t, ringtest.MustPoseidonHash(t, owner, nullifierPk), blinding)
	chunks := protocol.MintChunks(mint)
	c.MintChunks[0], c.MintChunks[1] = chunks[0], chunks[1]
	tree := ringregistry.NewTracking(t)
	tree.Apply(t, ringregistry.TrackingEntry{Member: owner, NullifierPk: nullifierPk, CtHash: big.NewInt(9),
		ListTag: big.NewInt(11), ViewingHash: viewingHash(t, recipientPk, 0)})
	c.TrackingRoot, c.Enrollment = tree.Root(), tree.Opening(owner).Wires()
	ringtest.SetBytes(c.OwnerKey[:], ownerKey[:])
	ringtest.SetBytes(c.RecipientPk[:], recipientPk[:])
	ringtest.SetBytes(c.TxViewingSk[:], txSk[:])
	ringtest.SetBytes(c.TxViewingPk[:], txPk[:])
	ringtest.SetBytes(c.AuditorPk[:], auditorPk[:])
	ringtest.SetBytes(c.Salt[:], salt[:])
	record := make([]byte, RecipientLen)
	record[0] = 1
	copy(record[9:41], ownerKey[:])
	copy(record[41:73], mint[:])
	blindingBytes := spptest.MustFieldBytes(t, blinding)
	copy(record[73:], blindingBytes[:])
	recordCt := ringprotocol.SealRecord(t, txSk, recipientPk, salt, index, context, RecipientLabel, record)
	ringtest.SetBytes(c.RecipientCiphertext[:], recordCt)
	secret := ringprotocol.DeliveredSlotSecret(t, txSk, recipientPk, salt, index, context)
	tag := ringprotocol.RecipientTag(t, secret)
	tagBytes := spptest.MustFieldBytes(t, tag)
	c.RecipientTag = new(big.Int).SetBytes(tagBytes[24:])
	// Exact host twin of official Envelope.SealSalted, with the new V1 label.
	tx := ringprotocol.XPrivateKey(t, txSk)
	peer, err := ecdh.X25519().NewPublicKey(auditorPk[:])
	if err != nil {
		t.Fatal(err)
	}
	dh, err := tx.ECDH(peer)
	if err != nil {
		t.Fatal(err)
	}
	dh0, dh1 := ringprotocol.Pack32([32]byte(dh))
	tx0, tx1 := ringprotocol.Pack33(ringprotocol.Tagged(txPk[:]))
	audit0, audit1 := ringprotocol.Pack33(ringprotocol.Tagged(auditorPk[:]))
	envelopeSecret := ringtest.MustPoseidonHash(t, big.NewInt(shared.DomSepCRShared), dh0, dh1, tx0, tx1, audit0, audit1)
	salted := ringtest.MustPoseidonHash(t, envelopeSecret, new(big.Int).SetBytes(salt[:]))
	key, nonce := ringprotocol.AuditStreamKeys(t, salted, AuditorLabel)
	auditCt := ringprotocol.CTREncrypt(t, key, nonce, slices.Concat(txSk[:], recipientPk[:]))
	ringtest.SetBytes(c.AuditorCiphertext[:], auditCt)
	rebind(t, c)
	return c
}

func rebind(t *testing.T, c *Circuit) {
	t.Helper()
	auditKey := make([]byte, 33)
	auditKey[0] = 4
	txKey := make([]byte, 33)
	txKey[0] = 4
	salt := make([]byte, 16)
	rc := make([]byte, RecipientLen)
	ac := make([]byte, AuditorLen)
	for i, v := range c.AuditorPk {
		auditKey[i+1] = byte(ringtest.AsBigInt(v).Uint64())
	}
	for i, v := range c.TxViewingPk {
		txKey[i+1] = byte(ringtest.AsBigInt(v).Uint64())
	}
	for i, v := range c.Salt {
		salt[i] = byte(ringtest.AsBigInt(v).Uint64())
	}
	for i, v := range c.RecipientCiphertext {
		rc[i] = byte(ringtest.AsBigInt(v).Uint64())
	}
	for i, v := range c.AuditorCiphertext {
		ac[i] = byte(ringtest.AsBigInt(v).Uint64())
	}
	a0, a1 := ringprotocol.Pack33([33]byte(auditKey))
	x0, x1 := ringprotocol.Pack33([33]byte(txKey))
	vals := []*big.Int{big.NewInt(Domain), big.NewInt(Version), ringtest.AsBigInt(c.RingID), a0, a1}
	fields := []interface{}{c.TrackingRoot, c.RingGeneration, c.OperationContext, c.OutputIndex,
		c.SourceAuthorityIdentity, c.PayerIdentity, c.TreeID, c.Asset, c.OwnerUtxoHash, c.RingDataHash, c.MinAmount, c.MaxAmount, c.ExpirySlot}
	for _, v := range fields {
		vals = append(vals, ringtest.AsBigInt(v))
	}
	vals = append(vals, x0, x1, new(big.Int).SetBytes(salt), ringtest.AsBigInt(c.RecipientTag), ringtest.HashBytes(t, rc), ringtest.HashBytes(t, ac))
	vals = append(vals, big.NewInt(int64(len(vals)+1)))
	if len(vals) != 25 {
		t.Fatalf("statement has %d elements", len(vals))
	}
	c.PublicInputHash = spptest.MustHashChain4(t, vals)
}

func TestDescriptorValidAndBinding(t *testing.T) {
	valid := fixture(t)
	if err := test.IsSolved(&Circuit{}, valid, ecc.BN254.ScalarField()); err != nil {
		t.Fatal(err)
	}
	cases := map[string]func(*Circuit){
		"owner commitment":                 func(c *Circuit) { c.OwnerUtxoHash = 17; rebind(t, c) },
		"wrong viewing sequence":           func(c *Circuit) { c.ViewingSeq = 1 },
		"wrong enrollment key":             func(c *Circuit) { c.NullifierPk = 123; rebind(t, c) },
		"wrong enrollment viewing key":     func(c *Circuit) { c.RecipientPk[0] = 17 },
		"wrong amount template ciphertext": func(c *Circuit) { c.RecipientCiphertext[1] = 17; rebind(t, c) },
		"wrong auditor":                    func(c *Circuit) { c.AuditorPk[0] = 17; rebind(t, c) },
		"wrong tag":                        func(c *Circuit) { c.RecipientTag = 17; rebind(t, c) },
		"wrong root":                       func(c *Circuit) { c.TrackingRoot = 17; rebind(t, c) },
		"arbitrary blinding":               func(c *Circuit) { c.Blinding = 17 },
		"wrong output index":               func(c *Circuit) { c.OutputIndex = 5; rebind(t, c) },
		"reversed amount bounds":           func(c *Circuit) { c.MinAmount = 10001; rebind(t, c) },
		"oversized amount":                 func(c *Circuit) { c.MaxAmount = new(big.Int).Lsh(big.NewInt(1), 64); rebind(t, c) },
		"unclamped scalar":                 func(c *Circuit) { c.TxViewingSk[0] = 1 },
		"source is original wallet":        func(c *Circuit) { c.SourceAuthorityIdentity = c.Enrollment.Record.Member; rebind(t, c) },
		"payer is original wallet":         func(c *Circuit) { c.PayerIdentity = c.Enrollment.Record.Member; rebind(t, c) },
		"context substitution":             func(c *Circuit) { c.OperationContext = 992; rebind(t, c) },
		"foreign ring data":                func(c *Circuit) { c.RingDataHash = 1; rebind(t, c) },
		"statement mutation":               func(c *Circuit) { c.ExpirySlot = 5001 },
	}
	for name, edit := range cases {
		t.Run(name, func(t *testing.T) {
			c := *valid
			edit(&c)
			if err := test.IsSolved(&Circuit{}, &c, ecc.BN254.ScalarField()); err == nil {
				t.Fatal("mutated descriptor accepted")
			}
		})
	}
}

func TestDescriptorNativeConstraintSystem(t *testing.T) {
	// Compile with the existing descriptor parameters, entirely in memory.
	// This is not trusted setup and must not create or load a key or proof.
	field := ecc.BN254.ScalarField()
	cs, err := frontend.Compile(field, r1cs.NewBuilder, &Circuit{}, frontend.WithCompressThreshold(300))
	if err != nil {
		t.Fatal(err)
	}
	commitments, ok := cs.GetCommitments().(constraint.Groth16Commitments)
	if !ok || len(commitments) != 1 || commitments[0].NbPublicCommitted != 0 {
		t.Fatal("descriptor must use exactly one private BSB22 commitment")
	}
	if cs.GetNbPublicVariables()-1 != 1 {
		t.Fatal("descriptor must expose exactly one public statement hash")
	}
	value := fixture(t)
	const historicalPublicHash = "5092304918366657430505280456797879521559189826696111908495058001243125583473"
	if ringtest.AsBigInt(value.PublicInputHash).String() != historicalPublicHash {
		t.Fatal("public statement differs from the synthetic descriptor-v1 fixture")
	}
	witness, err := frontend.NewWitness(value, field)
	if err != nil {
		t.Fatal(err)
	}
	if err := cs.IsSolved(witness); err != nil {
		t.Fatal("native descriptor rejects its synthetic fixture", err)
	}
	value.PublicInputHash = new(big.Int).Add(ringtest.AsBigInt(value.PublicInputHash), big.NewInt(1))
	wrong, err := frontend.NewWitness(value, field)
	if err != nil {
		t.Fatal(err)
	}
	if err := cs.IsSolved(wrong); err == nil {
		t.Fatal("native descriptor accepts a different public statement")
	}
	digest := sha256.New()
	if _, err := cs.WriteTo(digest); err != nil {
		t.Fatal(err)
	}
	t.Logf("constraints=%d public_inputs=%d private_commitments=%d r1cs_sha256=%x",
		cs.GetNbConstraints(), cs.GetNbPublicVariables()-1, len(commitments), digest.Sum(nil))
}

func TestRestrictedMaterialization(t *testing.T) {
	c := fixture(t)
	template := make([]byte, RecipientLen)
	for i, v := range c.RecipientCiphertext {
		template[i] = byte(ringtest.AsBigInt(v).Uint64())
	}
	var amount [8]byte
	binary.BigEndian.PutUint64(amount[:], 9237)
	final := slices.Clone(template)
	for i, b := range amount {
		final[i+1] ^= b
	}
	for i := range final {
		if i < 1 || i >= 9 {
			if final[i] != template[i] {
				t.Fatalf("byte %d changed", i)
			}
		}
	}
	for i, b := range amount {
		if final[i+1]^template[i+1] != b {
			t.Fatal("wrong CTR delta")
		}
	}
	txSk := ringprotocol.Clamped(ringtest.Scalar(0x73))
	recipientPk := ringprotocol.XPublicKey(t, ringprotocol.Clamped(ringtest.Scalar(0x72)))
	opened := ringprotocol.SealRecord(t, txSk, recipientPk, [16]byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16}, 1, big.NewInt(991), RecipientLabel, final)
	if opened[0] != 1 || binary.BigEndian.Uint64(opened[1:9]) != 9237 {
		t.Fatal("materialized record does not decrypt to actual amount")
	}
	expectedOwner := ringtest.Repeat(0x71)
	if !slices.Equal(opened[9:41], expectedOwner[:]) {
		t.Fatal("materialization changed owner")
	}
}

func TestDescriptorBoundaryOrdinals(t *testing.T) {
	for _, index := range []int{0, 4} {
		t.Run(big.NewInt(int64(index)).String(), func(t *testing.T) {
			c := fixtureAt(t, index)
			if err := test.IsSolved(&Circuit{}, c, ecc.BN254.ScalarField()); err != nil {
				t.Fatal(err)
			}
		})
	}
}

func viewingHash(t *testing.T, u [32]byte, seq uint64) *big.Int {
	a, b := ringprotocol.Pack32(u)
	return ringtest.MustPoseidonHash(t, big.NewInt(0x43525648), a, b, new(big.Int).SetUint64(seq))
}
