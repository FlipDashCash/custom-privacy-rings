use borsh::{BorshDeserialize, BorshSerialize};
use spr_program::{
    circuit::{
        require_eq, CheckedTransaction, ConfidentialTransaction, DataUtxo, PublicInputs,
        TokenUtxos, Utxo, UtxoTrait,
    },
    input, ArrayVec, CircuitError, Field, ShieldedAddress, TxContext, UtxoOwner, UtxoState,
    ZkProgram,
};

pub const TOKEN_INPUTS: usize = 2;

#[input]
#[derive(
    Default, PartialEq, Eq, BorshSerialize, BorshDeserialize, UtxoState, spr_program::ClientInputs,
)]
pub struct Counter {
    pub count: u64,
}

#[input]
#[derive(PublicInputs)]
pub struct OwnerPublicInputs {
    pub owner: UtxoOwner,
    pub ring: Field,
}

#[input]
#[derive(ZkProgram)]
pub struct Create {
    pub private: CreatePrivateInputs,
    pub public: OwnerPublicInputs,
}

#[input]
#[derive(spr_program::ClientInputs)]
pub struct CreatePrivateInputs {
    pub tx_context: TxContext,
    pub tokens: ArrayVec<Utxo, TOKEN_INPUTS>,
}

impl Create {
    pub fn execute(&self) -> Result<CheckedTransaction, CircuitError> {
        let private = &self.private;
        let ring = &self.public.ring;
        // 1. A note of the program on the ring leads the inputs, a state note has none before it.
        let tokens = TokenUtxos::new_mut_on(ring, &private.tokens)?.with_zero_output();
        require_eq(tokens.owner(), self.public.owner);
        // 2. The counter starts at zero.
        let state =
            DataUtxo::<Counter>::new_init(&self.public.owner).with_asset(&tokens.asset())?;

        // 3. The ring seals the change first on both rails.
        ConfidentialTransaction::new(&private.tx_context, &self.public)
            .with_ring(*ring)
            .with(tokens)
            .with(state)
            .check()
    }
}

#[input]
#[derive(ZkProgram)]
pub struct Update {
    pub private: UpdatePrivateInputs,
    pub public: OwnerPublicInputs,
}

#[input]
#[derive(spr_program::ClientInputs)]
pub struct UpdatePrivateInputs {
    pub tx_context: TxContext,
    pub state: Utxo,
    pub counter: Counter,
    pub tokens: ArrayVec<Utxo, TOKEN_INPUTS>,
    pub recipient: ShieldedAddress,
    pub amount: u64,
}

impl Update {
    pub fn execute(&self) -> Result<CheckedTransaction, CircuitError> {
        let private = &self.private;
        let ring = &self.public.ring;
        // 1. The spent notes sit on the ring and belong to the program.
        let mut state = DataUtxo::new_mut_on(ring, &private.state, &private.counter)?;
        let mut tokens = TokenUtxos::new_mut_on(ring, &private.tokens)?.with_zero_output();
        require_eq(state.owner(), self.public.owner);
        require_eq(tokens.owner(), self.public.owner);
        // 2. One step per update.
        state.count = private.counter.count + 1;
        // 3. A zero amount leaves the last slot empty.
        let mut payment = TokenUtxos::new_init(&private.recipient, &tokens.asset());
        tokens.transfer(&mut payment, &private.amount)?;

        // 4. The ring seals the change first on both rails.
        ConfidentialTransaction::new(&private.tx_context, &self.public)
            .with_ring(*ring)
            .with(tokens)
            .with(state)
            .with(payment)
            .check()
    }
}
