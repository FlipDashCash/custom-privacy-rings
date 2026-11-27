package ringregistry_test

import (
	"math/big"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/test"

	"github.com/helius-labs/zolana-custom-rings/prover/circuits/registry"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest/ringregistry"
	"github.com/helius-labs/zolana/prover/server/prover-test/spp/spptest"
)

func solvePrivate(c *registry.PrivateApplyRecordsCircuit) error {
	return test.IsSolved(c, c, ecc.BN254.ScalarField())
}

func TestPrivateWriteOpensTheOwnersTrackingRow(t *testing.T) {
	for _, mask := range []int64{ringregistry.PrivateBlockBit, ringregistry.PrivateAllowBit, ringregistry.PrivateFrozenBit} {
		if err := solvePrivate(ringregistry.PrivateBatch(t, 2, mask)); err != nil {
			t.Fatal(err)
		}
	}
	const owner = "registry.PrivateOwner.Assert"
	for _, tc := range []struct {
		name   string
		mutate func(*registry.PrivateApplyRecordsCircuit)
		frame  string
	}{
		{"another tracking root", func(c *registry.PrivateApplyRecordsCircuit) {
			c.TrackingRoot = ringregistry.NewTracking(t).Root()
		}, owner},
		{"another salt", func(c *registry.PrivateApplyRecordsCircuit) { c.Owners[0].ListSalt = 0x1234 }, owner},
		{"another owner's row", func(c *registry.PrivateApplyRecordsCircuit) {
			c.Owners[0].Record.Member = 103
		}, owner},
		{"an owner absent from the root", func(c *registry.PrivateApplyRecordsCircuit) {
			other := ringregistry.SaltedTracking(t, big.NewInt(13))
			other.Apply(t, ringregistry.TrackingEntry{
				Member: big.NewInt(103), NullifierPk: spptest.MustNullifierPk(t, big.NewInt(9)), CtHash: big.NewInt(19),
				ListTag: ringregistry.ListTag(t, big.NewInt(13)),
			})
			opening := other.Opening(big.NewInt(103))
			c.Owners[0].Record, c.Owners[0].Path = opening.Record.Wires(), opening.Path.Wires()
		}, owner},
		{"the sentinel", func(c *registry.PrivateApplyRecordsCircuit) {
			opening := ringregistry.NewTracking(t).Opening(big.NewInt(1))
			c.Owners[0].Record, c.Owners[0].Path = opening.Record.Wires(), opening.Path.Wires()
		}, owner},
		{"a bit past count", func(c *registry.PrivateApplyRecordsCircuit) {
			c.PrivateWrites = 3
			c.Owners[1] = c.Owners[0]
		}, "registry.(*PrivateApplyRecordsCircuit).Define"},
		{"an empty bitmap", func(c *registry.PrivateApplyRecordsCircuit) {
			c.PrivateWrites = 0
			c.Owners[0] = ringregistry.ZeroPrivateOwner()
		}, "registry.(*PrivateApplyRecordsCircuit).privateBits"},
	} {
		t.Run(tc.name, func(t *testing.T) {
			c := ringregistry.PrivateBatch(t, 2, ringregistry.PrivateBlockBit)
			tc.mutate(c)
			ringregistry.BindPrivateBatch(t, c)
			ringtest.RequireRefused(t, solvePrivate(c), tc.frame)
		})
	}
}

func TestPrivateKeyRefusesAPublicMask(t *testing.T) {
	ringtest.RequireRefused(t, solvePrivate(ringregistry.PrivateBatch(t, 2, ringregistry.AllowBit)), "registry.(*PrivateApplyRecordsCircuit).Define")
}

// A write under the ring domain needs no enrolled row and no tag, a write under the owner's salt keeps both.
func TestPrivateWriteUnderTheRingDomain(t *testing.T) {
	ringWrite := func(member, salt int64) *registry.PrivateApplyRecordsCircuit {
		return ringregistry.PrivateBatchFor(t, 2, ringregistry.PrivateBlockBit, ringregistry.NewRingOwnerWrite(t, member, big.NewInt(salt)))
	}
	for _, member := range []int64{101, 103} {
		if err := solvePrivate(ringWrite(member, 0x5a17)); err != nil {
			t.Fatal(err)
		}
	}
	const owner = "registry.PrivateOwner.Assert"
	for _, tc := range []struct {
		name   string
		batch  func() *registry.PrivateApplyRecordsCircuit
		mutate func(*registry.PrivateApplyRecordsCircuit)
	}{
		{"an absent owner keyed under its own salt domain", func() *registry.PrivateApplyRecordsCircuit { return ringWrite(103, 0x5a17) }, func(c *registry.PrivateApplyRecordsCircuit) {
			c.Owners[0].Ring = 0
		}},
		{"a ring key claimed for the owner's salt domain", func() *registry.PrivateApplyRecordsCircuit { return ringWrite(101, 13) }, func(c *registry.PrivateApplyRecordsCircuit) {
			c.Owners[0].Ring = 0
		}},
		{"an own salted key claimed for the ring domain", func() *registry.PrivateApplyRecordsCircuit {
			return ringregistry.PrivateBatch(t, 2, ringregistry.PrivateBlockBit)
		}, func(c *registry.PrivateApplyRecordsCircuit) {
			c.Owners[0].Ring = 1
		}},
		{"an absent owner called present", func() *registry.PrivateApplyRecordsCircuit { return ringWrite(103, 0x5a17) }, func(c *registry.PrivateApplyRecordsCircuit) {
			c.Owners[0].Present = 1
		}},
		{"an enrolled owner called absent", func() *registry.PrivateApplyRecordsCircuit { return ringWrite(101, 0x5a17) }, func(c *registry.PrivateApplyRecordsCircuit) {
			c.Owners[0].Present = 0
		}},
		{"another owner behind the ring key", func() *registry.PrivateApplyRecordsCircuit { return ringWrite(103, 0x5a17) }, func(c *registry.PrivateApplyRecordsCircuit) {
			c.Owners[0].Owner = 104
		}},
	} {
		t.Run(tc.name, func(t *testing.T) {
			c := tc.batch()
			tc.mutate(c)
			ringtest.RequireRefused(t, solvePrivate(c), owner)
		})
	}
}
