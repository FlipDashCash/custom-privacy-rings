//go:build !pyrol_delivery_localnet

package ring

// No unqualified delivery key is registered in the default prover.
func deliveryLocalKeys() []Key { return nil }
