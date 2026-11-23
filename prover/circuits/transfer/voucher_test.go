package transfer_test

import (
	"math/big"
	"testing"

	"github.com/helius-labs/zolana-custom-rings/prover/ringtest"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest/ringaudit"
	"github.com/helius-labs/zolana-custom-rings/prover/ringtest/ringtransfer"
)

func vouchedWorld() ringtransfer.Confidential {
	w := ringtransfer.Signed(confidential2)
	w.Voucher, w.Unlisted = ringaudit.VoucherKey, true
	return w
}

// One listed signer admits a sender and owners no allow entry names, and nobody passes without it.
func TestVoucherAdmitsUnlistedOwners(t *testing.T) {
	requireSolved(t, vouchedWorld().Build(t).Circuit)
	t.Run("no voucher", func(t *testing.T) {
		w := vouchedWorld()
		w.Voucher = nil
		refused(t, w.Build(t).Circuit, "policy.assertPaymentRules")
	})
	t.Run("a throwaway key in the listed voucher's place", func(t *testing.T) {
		b := vouchedWorld().Build(t)
		unvouched := b.Audit
		unvouched.Voucher = nil
		b.Circuit.Voucher = unvouched.VoucherWires(t, ringtest.AsBigInt(b.Circuit.PrivateTxHash), ringtest.AsBigInt(b.Circuit.WithdrawalHash))
		refused(t, b.Circuit)
	})
}

// A signature that fails the curve equation leaves the gadget's hinted limbs out of range, the refusal surfaces where the range table closes.
const rangeTable = "nonnative.(*rangeTable).build"

// A vouch is a signature over one transfer's hashes by the key the proof names.
func TestVouchBindsTheTransfer(t *testing.T) {
	hashes := func(b ringtransfer.Built) (privateTxHash, withdrawalHash *big.Int) {
		return ringtest.AsBigInt(b.Circuit.PrivateTxHash), ringtest.AsBigInt(b.Circuit.WithdrawalHash)
	}
	t.Run("a signature over another transfer", func(t *testing.T) {
		b := vouchedWorld().Build(t)
		privateTxHash, withdrawalHash := hashes(b)
		b.Circuit.Voucher = b.Audit.VoucherWires(t, new(big.Int).Add(privateTxHash, big.NewInt(1)), withdrawalHash)
		refused(t, b.Circuit, rangeTable)
	})
	t.Run("a signature over other withdrawal legs", func(t *testing.T) {
		b := vouchedWorld().Build(t)
		privateTxHash, withdrawalHash := hashes(b)
		b.Circuit.Voucher = b.Audit.VoucherWires(t, privateTxHash, new(big.Int).Add(withdrawalHash, big.NewInt(1)))
		refused(t, b.Circuit, rangeTable)
	})
	t.Run("another key's signature", func(t *testing.T) {
		b := vouchedWorld().Build(t)
		unvouched := b.Audit
		unvouched.Voucher = nil
		privateTxHash, withdrawalHash := hashes(b)
		b.Circuit.Voucher.Signature = unvouched.VoucherWires(t, privateTxHash, withdrawalHash).Signature
		refused(t, b.Circuit, rangeTable)
	})
	t.Run("an unvouched transfer under a forged signature", func(t *testing.T) {
		b := ringtransfer.Signed(confidential2).Build(t)
		b.Circuit.Voucher.Signature[0] = ringtest.AsBigInt(b.Circuit.Voucher.Signature[0]).Uint64() ^ 1
		refused(t, b.Circuit, "ed25519.Verify")
	})
}
