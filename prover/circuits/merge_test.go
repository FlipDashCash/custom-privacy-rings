package circuits_test

import (
	"math/big"
	"slices"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/test"

	"github.com/helius-labs/zolana-custom-rings/prover/circuits"
	"github.com/helius-labs/zolana-custom-rings/prover/circuits/audit"
	"github.com/helius-labs/zolana-custom-rings/prover/circuits/shared"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest/ringaudit"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest/ringkey"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest/ringmerge"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest/ringprotocol"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest/ringregistry"
	mergeshared "github.com/helius-labs/zolana/prover/server/circuits/spp_merge/shared"
	"github.com/helius-labs/zolana/prover/server/prover-test/spp/protocol"
	"github.com/helius-labs/zolana/prover/server/prover-test/spp/spptest"
)

func solved(t *testing.T, s *ringmerge.Statement) error {
	t.Helper()
	return test.IsSolved(&circuits.MergeCircuit{}, s.Circuit, ecc.BN254.ScalarField())
}

// The statement stays the program's, each case changes what the prover claims behind it.
func TestMergeRefusals(t *testing.T) {
	define := []string{"[assertIsEqual]", "circuits.(*MergeCircuit).Define"}
	tests := []struct {
		name   string
		tamper func(*ringmerge.Statement)
	}{
		{"another amount", func(s *ringmerge.Statement) { s.Circuit.Amount = big.NewInt(13) }},
		{"another owner key", func(s *ringmerge.Statement) { s.Circuit.OwnerKey[0] = 0 }},
		{"another mint", func(s *ringmerge.Statement) { s.Circuit.MintChunks[1] = 0 }},
		{"another asset", func(s *ringmerge.Statement) { s.Circuit.Asset = 1 }},
		{"another owner identity", func(s *ringmerge.Statement) { s.Circuit.OwnerPkHash = 1 }},
		{"another seed", func(s *ringmerge.Statement) { s.Circuit.Seed = big.NewInt(20) }},
		{"the zero seed", func(s *ringmerge.Statement) { s.Circuit.Seed = big.NewInt(0) }},
		{"another nullifier key", func(s *ringmerge.Statement) { s.Circuit.NullifierPk = big.NewInt(5) }},
		{"another ring", func(s *ringmerge.Statement) { s.Circuit.RingProgramID = big.NewInt(0x5a0f) }},
		{"another output tree", func(s *ringmerge.Statement) { s.Circuit.OutputTreeID = 10 }},
		{"another ring data hash", func(s *ringmerge.Statement) { s.Circuit.OutputRingDataHash = 1 }},
		{"another first nullifier", func(s *ringmerge.Statement) { s.Circuit.FirstNullifier = big.NewInt(1) }},
		{"another auditor", func(s *ringmerge.Statement) { s.Circuit.AuditorPk[1] = 0 }},
		{"a ciphertext of another record", func(s *ringmerge.Statement) {
			s.Record = slices.Clone(s.Record)
			s.Record[15] ^= 1
			s.Rebind(t)
		}},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			s := ringmerge.Build(t)
			tt.tamper(s)
			ringtest.RequireRefused(t, solved(t, s), define...)
		})
	}
}

// A witness off the record's opening refuses inside the record encoder.
func TestMergeRecordOffItsOpening(t *testing.T) {
	for name, tamper := range map[string]func(*ringmerge.Statement){
		"another owner key": func(s *ringmerge.Statement) { s.Circuit.OwnerKey[0] = 0 },
		"another mint":      func(s *ringmerge.Statement) { s.Circuit.MintChunks[1] = 0 },
	} {
		t.Run(name, func(t *testing.T) {
			s := ringmerge.Build(t)
			tamper(s)
			ringtest.RequireRefused(t, solved(t, s), "audit.MergeRecord.Encode")
		})
	}
	type flip struct {
		offset int
		mask   byte
	}
	for name, flip := range map[string]flip{
		"a sealed seed off the opening":     {audit.RecordLen, 1},
		"a sealed blinding off the opening": {audit.RingRecordLen - 1, 1},
		"a sealed key off the opening":      {audit.RecordLen - 1, 1},
		"a forged zero key bit":             {0, audit.KindZeroKey},
	} {
		t.Run(name, func(t *testing.T) {
			s := ringmerge.Build(t)
			s.Record = slices.Clone(s.Record)
			s.Record[flip.offset] ^= flip.mask
			s.Rebind(t)
			ringtest.RequireRefused(t, solved(t, s), "circuits.(*MergeCircuit).Define")
		})
	}
}

// The merge record is sealed to the recipient key its envelope gives the auditor.
func TestMergeRecordIsSealedToTheEnvelopesKey(t *testing.T) {
	s := ringmerge.Build(t)
	if err := solved(t, s); err != nil {
		t.Fatal(err)
	}
	t.Run("the owner opens the record under its viewing key", func(t *testing.T) {
		record := ringprotocol.SealRecord(t, s.Keys.EphSk(), s.RecipientKey, [16]byte{}, 0, ringmerge.FirstNullifier, shared.MergeOutputInfo, s.Ciphertext(t))
		if !slices.Equal(record, s.Record) {
			t.Fatal("the record differs from the merged opening")
		}
	})
	t.Run("an envelope naming another key", func(t *testing.T) {
		s := ringmerge.Build(t)
		ringtest.SetBytes(s.Circuit.RecipientKey[:], func() []byte { key := ringaudit.ViewingKey(t, 0x41); return key[:] }())
		ringtest.RequireRefused(t, solved(t, s), "circuits.(*MergeCircuit).Define")
	})
	t.Run("a record under the transfer label", func(t *testing.T) {
		s := ringmerge.Build(t)
		transferRecord := ringprotocol.SealRecord(t, s.Keys.EphSk(), s.RecipientKey, [16]byte{}, 0, ringmerge.FirstNullifier, shared.OutputRecordInfo, s.Record)
		if slices.Equal(transferRecord, s.Ciphertext(t)) {
			t.Fatal("the merge record shares the transfer keystream")
		}
	})
}

// A byte outside 0..255 packing to the same identity would open another record.
func TestMergeOwnerKeyBytesStayBytes(t *testing.T) {
	s := ringmerge.Build(t)
	key := &s.Circuit.OwnerKey
	key[0], key[1] = ringtest.AsBigInt(key[0]).Int64()-1, ringtest.AsBigInt(key[1]).Int64()+256
	ringtest.RequireRefused(t, solved(t, s), "lookup query too large")
}

// The record's amount is eight bytes, a wider amount has no record.
func TestMergeRefusesAnAmountAboveItsBytes(t *testing.T) {
	s := ringmerge.Build(t)
	s.Amount = new(big.Int).Lsh(big.NewInt(1), 64)
	s.Circuit.Amount = s.Amount
	s.OutputHash = spptest.MustUtxoHash(t, protocol.Utxo{
		Domain: big.NewInt(protocol.UtxoDomain), Owner: spptest.MustOwnerHash(t, s.OwnerPkHash, s.NullifierPk), Asset: ringtest.AssetField(t, s.Mint), Amount: s.Amount,
		Blinding: s.OutputBlinding, DataHash: big.NewInt(0), RingDataHash: ringmerge.RingData, RingProgramID: ringmerge.RingProgramID,
	}, big.NewInt(ringmerge.OutputTreeID))
	s.Circuit.OutputUtxoHash = s.OutputHash
	s.Rebind(t)
	ringtest.RequireRefused(t, solved(t, s), "[ToBinary]", "audit.RecordOpening.Encode")
}

func TestMergeTrackingRow(t *testing.T) {
	tests := []struct {
		name   string
		tamper func(*testing.T, *ringmerge.Statement)
	}{
		{"another root", func(t *testing.T, s *ringmerge.Statement) {
			s.Circuit.TrackingRoot = big.NewInt(1)
			s.Rebind(t)
		}},
		{"another row field", func(_ *testing.T, s *ringmerge.Statement) {
			s.Circuit.Tracking.Record.CtHash = big.NewInt(1)
		}},
		{"another path", func(_ *testing.T, s *ringmerge.Statement) {
			s.Circuit.Tracking.Path.Index = 2
		}},
		{"another member's row", func(t *testing.T, s *ringmerge.Statement) {
			other := big.NewInt(0x77)
			s.Tracking.Apply(t, s.TrackingEntry(other, s.NullifierPk))
			s.Circuit.Tracking = s.Tracking.Opening(other).Wires()
			s.Circuit.TrackingRoot = s.Tracking.Root()
			s.Rebind(t)
		}},
		{"a row under another key", func(t *testing.T, s *ringmerge.Statement) {
			s.Tracking = ringregistry.NewTracking(t)
			s.Tracking.Apply(t, s.TrackingEntry(s.OwnerPkHash, big.NewInt(5)))
			s.Circuit.Tracking = s.Tracking.Opening(s.OwnerPkHash).Wires()
			s.Circuit.TrackingRoot = s.Tracking.Root()
			s.Rebind(t)
		}},
		{"an absent row", func(t *testing.T, s *ringmerge.Statement) {
			s.Tracking = ringregistry.NewTracking(t)
			s.Tracking.Apply(t, s.TrackingEntry(big.NewInt(0x77), s.NullifierPk))
			s.Circuit.Tracking = s.Tracking.Opening(s.OwnerPkHash).Wires()
			s.Circuit.TrackingRoot = s.Tracking.Root()
			s.Rebind(t)
		}},
		{"no row at all", func(t *testing.T, s *ringmerge.Statement) {
			zero := big.NewInt(0)
			s.Circuit.Tracking = ringregistry.TrackingOpening{
				Record: ringregistry.TrackingRecord{Member: zero, Next: zero, NullifierPk: zero, CtHash: zero, ListTag: zero},
				Path:   ringregistry.ZeroTrackingWrite().Slot,
			}.Wires()
			s.Circuit.TrackingRoot = 0
			s.Rebind(t)
		}},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			s := ringmerge.Build(t)
			tt.tamper(t, s)
			ringtest.RequireRefused(t, solved(t, s), "[assertIsEqual]", "circuits.(*MergeCircuit).Define")
		})
	}
}

// rekey rebuilds the blinding, the record and SPP's ciphertext under seed and nullifierPk, the output hash stays SPP's.
func rekey(t *testing.T, s *ringmerge.Statement, seed, nullifierPk *big.Int) {
	t.Helper()
	s.Seed, s.NullifierPk = seed, nullifierPk
	s.Circuit.Seed, s.Circuit.NullifierPk = seed, nullifierPk
	s.OutputBlinding = ringtest.MustPoseidonHash(t, big.NewInt(mergeshared.MergeOutputBlindingDomainV1), seed, ringmerge.FirstNullifier)
	s.Record = s.PlainRecord(t)
	s.OutputCiphertext = s.SealOutput(t)
}

// SPP's output hash holds the seed to the nullifier secret, a witness rebuilt under another seed opens another output.
func TestMergeSeedIsTheNullifierSecret(t *testing.T) {
	s := ringmerge.Build(t)
	rekey(t, s, big.NewInt(20), s.NullifierPk)
	s.Rebind(t)
	ringtest.RequireRefused(t, solved(t, s), "[assertIsEqual]", "circuits.(*MergeCircuit).Define")
}

// register_key.go enrolls no zero key, a zero key witness solves under a row and refuses without one.
func TestMergeRefusesTheZeroKey(t *testing.T) {
	s := ringmerge.Build(t)
	rekey(t, s, big.NewInt(0), shared.ZeroNullifierPk)
	s.Record[0] |= audit.KindZeroKey
	s.OutputHash = ringmerge.Open(t, s).Hash(t)
	s.Circuit.OutputUtxoHash = s.OutputHash
	enroll := func(member *big.Int) {
		s.Tracking = ringregistry.NewTracking(t)
		s.Tracking.Apply(t, s.TrackingEntry(member, s.NullifierPk))
		s.Circuit.Tracking = s.Tracking.Opening(s.OwnerPkHash).Wires()
		s.Circuit.TrackingRoot = s.Tracking.Root()
		s.Rebind(t)
	}
	enroll(s.OwnerPkHash)
	if err := solved(t, s); err != nil {
		t.Fatal(err)
	}
	enroll(big.NewInt(0x77))
	ringtest.RequireRefused(t, solved(t, s), "[assertIsEqual]", "circuits.(*MergeCircuit).Define")
}

// SPP publishes its output ciphertext unproven, the statement holds it to the witness seed, amount, mint and salt.
func TestMergeOutputCiphertext(t *testing.T) {
	t.Run("the owner opens the amount and mint", func(t *testing.T) {
		s := ringmerge.Build(t)
		if err := solved(t, s); err != nil {
			t.Fatal(err)
		}
		key, err := protocol.MergeOutputKey(s.Seed, ringmerge.FirstNullifier)
		if err != nil {
			t.Fatal(err)
		}
		chunks := protocol.MintChunks(s.Mint)
		for element, want := range []*big.Int{s.Amount, chunks[0], chunks[1]} {
			pad := ringtest.MustPoseidonHash(t, big.NewInt(protocol.MergeOutputCiphertextDomain), key, ringmerge.OutputCiphertextSalt, big.NewInt(int64(element)))
			opened := new(big.Int).Sub(s.OutputCiphertext[element], pad)
			if opened.Mod(opened, ecc.BN254.ScalarField()).Cmp(want) != 0 {
				t.Fatalf("element %d opens to %s", element, opened)
			}
		}
	})
	t.Run("a fresh salt reseals the output", func(t *testing.T) {
		s := ringmerge.Build(t)
		s.Circuit.OutputCiphertextSalt = big.NewInt(0x5a18)
		s.OutputCiphertext = s.SealOutput(t)
		s.Rebind(t)
		if err := solved(t, s); err != nil {
			t.Fatal(err)
		}
	})
	sealed := func(t *testing.T, s *ringmerge.Statement, amount *big.Int, mint [32]byte) []*big.Int {
		t.Helper()
		ciphertext, err := protocol.MergeOutputCiphertext(s.Seed, ringmerge.FirstNullifier, ringmerge.OutputCiphertextSalt, amount, mint)
		if err != nil {
			t.Fatal(err)
		}
		return ciphertext
	}
	tests := []struct {
		name   string
		tamper func(*testing.T, *ringmerge.Statement)
	}{
		{"another amount element", func(t *testing.T, s *ringmerge.Statement) {
			s.OutputCiphertext[0] = new(big.Int).Add(s.OutputCiphertext[0], big.NewInt(1))
			s.Rebind(t)
		}},
		{"another first mint element", func(t *testing.T, s *ringmerge.Statement) {
			s.OutputCiphertext[1] = new(big.Int).Add(s.OutputCiphertext[1], big.NewInt(1))
			s.Rebind(t)
		}},
		{"another second mint element", func(t *testing.T, s *ringmerge.Statement) {
			s.OutputCiphertext[2] = new(big.Int).Add(s.OutputCiphertext[2], big.NewInt(1))
			s.Rebind(t)
		}},
		{"the encryption of another amount", func(t *testing.T, s *ringmerge.Statement) {
			s.OutputCiphertext = sealed(t, s, big.NewInt(13), s.Mint)
			s.Rebind(t)
		}},
		{"the encryption of another mint", func(t *testing.T, s *ringmerge.Statement) {
			mint := s.Mint
			mint[0] ^= 1
			s.OutputCiphertext = sealed(t, s, s.Amount, mint)
			s.Rebind(t)
		}},
		{"the encryption under another seed", func(t *testing.T, s *ringmerge.Statement) {
			ciphertext, err := protocol.MergeOutputCiphertext(big.NewInt(20), ringmerge.FirstNullifier, ringmerge.OutputCiphertextSalt, s.Amount, s.Mint)
			if err != nil {
				t.Fatal(err)
			}
			s.OutputCiphertext = ciphertext
			s.Rebind(t)
		}},
		{"a witness salt off the statement", func(_ *testing.T, s *ringmerge.Statement) {
			s.Circuit.OutputCiphertextSalt = big.NewInt(0x5a18)
		}},
		{"a statement salt off the witness", func(t *testing.T, s *ringmerge.Statement) {
			s.Circuit.OutputCiphertextSalt = big.NewInt(0x5a18)
			s.OutputCiphertext = s.SealOutput(t)
			s.Rebind(t)
			s.Circuit.OutputCiphertextSalt = ringmerge.OutputCiphertextSalt
		}},
		{"a ciphertext under another salt", func(t *testing.T, s *ringmerge.Statement) {
			s.Circuit.OutputCiphertextSalt = big.NewInt(0x5a18)
			s.Rebind(t)
		}},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			s := ringmerge.Build(t)
			tt.tamper(t, s)
			ringtest.RequireRefused(t, solved(t, s), "[assertIsEqual]", "circuits.(*MergeCircuit).Define")
		})
	}
}

func TestMergePayer(t *testing.T) {
	// Paying names the hidden owner, a privacy choice the wallet makes and no proof refuses.
	t.Run("a hidden owner pays", func(t *testing.T) {
		s := ringmerge.Build(t)
		s.Circuit.Payer = s.OwnerPkHash
		s.Rebind(t)
		if err := solved(t, s); err != nil {
			t.Fatal(err)
		}
	})
	// The authorization stays, a paying hidden owner still opens its own row under its own key.
	t.Run("a paying hidden owner under another key", func(t *testing.T) {
		s := ringmerge.Build(t)
		s.Circuit.Payer = s.OwnerPkHash
		s.Tracking = ringregistry.NewTracking(t)
		s.Tracking.Apply(t, s.TrackingEntry(s.OwnerPkHash, big.NewInt(5)))
		s.Circuit.Tracking = s.Tracking.Opening(s.OwnerPkHash).Wires()
		s.Circuit.TrackingRoot = s.Tracking.Root()
		s.Rebind(t)
		ringtest.RequireRefused(t, solved(t, s), "circuits.(*MergeCircuit).Define")
	})
	t.Run("the payer is statement bound", func(t *testing.T) {
		s := ringmerge.Build(t)
		s.Circuit.Payer = big.NewInt(99)
		if solved(t, s) == nil {
			t.Fatal("unbound payer accepted")
		}
	})
}

// The record alone rebuilds the output.
func TestMergeRecordRebuildsTheOutput(t *testing.T) {
	s := ringmerge.Build(t)
	opened := ringmerge.Open(t, s)
	if len(opened.Plain) != circuits.MergeRecordLen {
		t.Fatalf("%d byte record", len(opened.Plain))
	}
	if opened.Hash(t).Cmp(s.OutputHash) != 0 {
		t.Fatal("the record does not rebuild the published output")
	}
}

func TestMergeCost(t *testing.T) {
	cs := ringkey.Compile(t, &circuits.MergeCircuit{})
	ringkey.RequirePinned(t, "custom_ring_merge", cs)
	if err := solveCS(t, cs, ringmerge.Build(t).Circuit); err != nil {
		t.Fatal(err)
	}
}

func solveCS(t *testing.T, cs constraint.ConstraintSystem, c *circuits.MergeCircuit) error {
	t.Helper()
	witness, err := frontend.NewWitness(c, ecc.BN254.ScalarField())
	if err != nil {
		t.Fatal(err)
	}
	_, err = cs.Solve(witness)
	return err
}
