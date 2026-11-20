// Package descriptor defines the versioned dynamic-deposit descriptor relation.
// It imports the pinned official gadgets; it does not modify their statements.
package descriptor

import (
	"slices"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/rangecheck"
	"github.com/helius-labs/zolana-custom-rings/prover/circuits/audit"
	"github.com/helius-labs/zolana-custom-rings/prover/circuits/registry"
	"github.com/helius-labs/zolana-custom-rings/prover/circuits/ringgadget"
	"github.com/helius-labs/zolana-custom-rings/prover/circuits/shared"
	"github.com/helius-labs/zolana/prover/server/circuits/crypto/x25519"
	"github.com/helius-labs/zolana/prover/server/circuits/gadget"
	transaction "github.com/helius-labs/zolana/prover/server/circuits/spp_transaction/shared"
)

const (
	Domain             = 0x43524444 // CRDD, not the ordinary CRTX transfer statement.
	Version            = 1
	RecipientLabel     = "CRDY1/rcp"
	AuditorLabel       = "CRDY1/adt"
	RecipientLen       = 105
	AuditorLen         = 64
	RecipientTagDomain = 0x43525254 // official CRRT
)

// Circuit has exactly one public signal, matching ordinary official Ring proofs.
// The processor reconstructs every statement preimage field from live accounts,
// bounded plan metadata and the explicit descriptor capsule before verification.
type Circuit struct {
	PublicInputHash                            frontend.Variable `gnark:",public"`
	RingID, TrackingRoot, RingGeneration       frontend.Variable
	OperationContext, OutputIndex              frontend.Variable
	SourceAuthorityIdentity, PayerIdentity     frontend.Variable
	TreeID, Asset, OwnerUtxoHash, RingDataHash frontend.Variable
	MinAmount, MaxAmount, ExpirySlot           frontend.Variable
	RecipientTag                               frontend.Variable
	AuditorPk                                  [32]frontend.Variable
	TxViewingPk                                [32]frontend.Variable
	Salt                                       [16]frontend.Variable
	RecipientCiphertext                        [RecipientLen]frontend.Variable
	AuditorCiphertext                          [AuditorLen]frontend.Variable

	// The wallet owner remains a circuit witness and encrypted record field.
	OwnerKey                          [32]frontend.Variable
	NullifierPk, Blinding, ViewingSeq frontend.Variable
	RecipientPk, TxViewingSk          [32]frontend.Variable
	MintChunks                        [2]frontend.Variable
	Enrollment                        registry.TrackingOpening
}

func (c *Circuit) Define(api frontend.API) error {
	rc := rangecheck.New(api)
	audit.AssertClamped(api, c.TxViewingSk)
	owner := gadget.SolanaOwnerIdentity(api, c.OwnerKey)
	api.AssertIsEqual(api.IsZero(api.Sub(owner, c.SourceAuthorityIdentity)), 0)
	api.AssertIsEqual(api.IsZero(api.Sub(owner, c.PayerIdentity)), 0)
	api.AssertIsEqual(c.Enrollment.Present, 1)
	row := c.Enrollment.Resolve(api, 1, c.TrackingRoot, owner)
	api.AssertIsEqual(row.NullifierPk, c.NullifierPk)
	api.AssertIsEqual(api.IsZero(c.NullifierPk), 0)
	api.AssertIsEqual(api.IsZero(api.Sub(c.NullifierPk, shared.ZeroNullifierPk)), 0)
	api.AssertIsEqual(c.RingDataHash, 0)
	api.AssertIsEqual(api.IsZero(c.OperationContext), 0)
	// The output ordinal is bounded to five settlement outputs (0 through 4).
	api.ToBinary(c.OutputIndex, 8)
	api.AssertIsLessOrEqual(c.OutputIndex, 4)
	for _, v := range []frontend.Variable{c.MinAmount, c.MaxAmount, c.ExpirySlot} {
		api.ToBinary(v, 64)
	}
	api.ToBinary(c.RingGeneration, 32)
	api.AssertIsLessOrEqual(c.MinAmount, c.MaxAmount)

	derivedPk := x25519.ScalarMulGenerator(api, c.TxViewingSk)
	for i := range derivedPk {
		api.AssertIsEqual(c.TxViewingPk[i], derivedPk[i])
	}
	eph := shared.PackedKey(api, derivedPk)
	salt := gadget.BytesToFieldWith(api, rc, c.Salt[:])
	packedSk := shared.PackBounded(api, c.TxViewingSk[:])
	seed := shared.BlindingSeed(api, [2]frontend.Variable{packedSk[0], packedSk[1]}, salt)
	outputSeed := transaction.DeriveOutputBlindingSeed(api, c.OperationContext, seed)
	// Official TXOB expression; its public ordinal is variable in this new shape.
	derivedBlinding := gadget.PoseidonHash(api, []frontend.Variable{transaction.OutputBlindingDomainV1, c.OperationContext, outputSeed, c.OutputIndex})
	api.AssertIsEqual(c.Blinding, derivedBlinding)
	api.AssertIsEqual(c.OwnerUtxoHash, gadget.PoseidonHash(api, []frontend.Variable{
		gadget.PoseidonHash(api, []frontend.Variable{owner, c.NullifierPk}), c.Blinding,
	}))

	fullRecord := (audit.RecordOpening{Live: 1, Kind: audit.KindRing, Amount: 0,
		OwnerPkHash: owner, NullifierPk: c.NullifierPk, Asset: c.Asset, Blinding: c.Blinding,
		OwnerKey: c.OwnerKey, MintChunks: c.MintChunks}).Encode(api, rc)
	record := shared.SlotSeal{Plaintext: fullRecord[:RecipientLen], EphSk: c.TxViewingSk, Eph: eph,
		RecipientPk: c.RecipientPk, Salt: salt, Index: c.OutputIndex, FirstNullifier: c.OperationContext,
		Info: RecipientLabel}.Seal(api)
	for i, v := range record {
		api.AssertIsEqual(c.RecipientCiphertext[i], v)
	}
	// Agreement constrains recipient key bytes and rejects a low-order key.
	agreement := shared.AgreeCanonical(api, c.TxViewingSk, c.RecipientPk)
	api.ToBinary(c.ViewingSeq, 64)
	viewingParts := shared.PackBounded(api, c.RecipientPk[:])
	// Exact official accounts/tracking.rs::viewing_hash and TS viewingKeyHash;
	// the older Go tracking comment incorrectly describes hash_bytes(u).
	api.AssertIsEqual(c.Enrollment.Record.ViewingHash, gadget.PoseidonHash(api, []frontend.Variable{0x43525648, viewingParts[0], viewingParts[1], c.ViewingSeq}))
	slotSecret := gadget.PoseidonHash(api, slices.Concat(
		[]frontend.Variable{shared.DeliveredSlotDomain}, shared.PackBounded(api, agreement[:]), eph,
		shared.PackedKey(api, c.RecipientPk), []frontend.Variable{salt, c.OutputIndex, c.OperationContext},
	))
	tagField := gadget.PoseidonHash(api, []frontend.Variable{RecipientTagDomain, slotSecret})
	tagBytes := ringgadget.FieldToBytesBEWith(api, rc, tagField)
	api.AssertIsEqual(c.RecipientTag, gadget.BytesToFieldWith(api, ringgadget.BoundedBytes{}, tagBytes[24:]))

	sealed := (shared.Envelope{Plaintext: slices.Concat(c.TxViewingSk[:], c.RecipientPk[:]),
		EphSk: c.TxViewingSk, AuditorPk: c.AuditorPk, Info: AuditorLabel}).SealSalted(api, eph, salt)
	for i, v := range sealed.Ciphertext {
		api.AssertIsEqual(c.AuditorCiphertext[i], v)
	}
	fields := slices.Concat([]frontend.Variable{Domain, Version, c.RingID}, sealed.Auditor,
		[]frontend.Variable{c.TrackingRoot, c.RingGeneration, c.OperationContext, c.OutputIndex,
			c.SourceAuthorityIdentity, c.PayerIdentity, c.TreeID, c.Asset, c.OwnerUtxoHash, c.RingDataHash,
			c.MinAmount, c.MaxAmount, c.ExpirySlot}, eph, []frontend.Variable{salt, c.RecipientTag,
			gadget.HashBoundedBytes(api, c.RecipientCiphertext[:]), gadget.HashBoundedBytes(api, c.AuditorCiphertext[:])})
	// Count includes its own final cell, matching official closeStatement style.
	fields = append(fields, len(fields)+1)
	api.AssertIsEqual(c.PublicInputHash, gadget.HashChain4(api, fields))
	return nil
}
