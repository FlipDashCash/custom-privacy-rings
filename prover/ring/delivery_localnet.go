//go:build pyrol_delivery_localnet

package ring

// Distinct, opt-in Localnet profile. Existing family/key names are preserved.
func deliveryLocalKeys() []Key {
	return []Key{
		{Family: DeliveryTransfer, Inputs: 24, Outputs: 4, Withdrawals: 4, Owners: 4},
		{Family: DeliveryTransfer, Inputs: 16, Outputs: 8, Withdrawals: 4, Owners: 4},
	}
}
