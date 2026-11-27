use std::{
    cell::{OnceCell, RefCell},
    collections::HashMap,
    path::PathBuf,
};

use anyhow::{ensure, Context as _, Result};
use custom_ring_interface::{RingRole, RingRoles};
use solana_address::Address;
use solana_hash::Hash;
use zolana_operator_custody::{
    KeyArn, Kms, KmsEndpoint, RpcCustody, SecretContext, SecretPurpose, SignerRef, SigningIdentity,
};

use crate::{
    config::{parse_json, OperatorManifest, Urls},
    context::cached,
    fund, Context, Target,
};

/// Read once per command, a signer costs one KMS round trip.
#[derive(Default)]
pub(crate) struct Custody {
    manifest: OnceCell<OperatorManifest>,
    kms: OnceCell<Kms>,
    signers: RefCell<HashMap<(Address, KeyArn), SigningIdentity>>,
    local_rpc: OnceCell<Option<RpcCustody>>,
}

impl Context {
    pub(crate) fn operator_manifest_path(&self) -> Result<PathBuf> {
        self.config.operator_manifest_path(&self.root)
    }

    pub(crate) fn operator_manifest(&self) -> Result<&OperatorManifest> {
        cached(&self.custody.manifest, || {
            OperatorManifest::load_for(&self.operator_manifest_path()?, self.ring.program_id())
        })
    }

    pub(crate) fn policy_address(&self) -> Result<Address> {
        Ok(self.operator_manifest()?.policy.address)
    }

    pub(crate) fn upgrade_authority_address(&self) -> Result<Address> {
        Ok(self.operator_manifest()?.upgrade_authority.address)
    }

    pub(crate) fn signer(&self, pin: &SignerRef) -> Result<SigningIdentity> {
        let key = (pin.address, pin.key_arn.clone());
        if let Some(signer) = self.custody.signers.borrow().get(&key) {
            return Ok(signer.clone());
        }
        let kms = self.kms()?;
        let signer = self.block(async { anyhow::Ok(kms.signer(pin).await?) })?;
        self.custody
            .signers
            .borrow_mut()
            .insert(key, signer.clone());
        Ok(signer)
    }

    pub(crate) fn program_signer(&self) -> Result<SigningIdentity> {
        self.signer(&self.operator_manifest()?.program)
    }

    pub(crate) fn upgrade_authority_signer(&self) -> Result<SigningIdentity> {
        self.signer(&self.operator_manifest()?.upgrade_authority)
    }

    pub(crate) fn delegate_signer(&self) -> Result<SigningIdentity> {
        self.signer(
            self.operator_manifest()?
                .delegate
                .as_ref()
                .context("the operator manifest names no delegate")?,
        )
    }

    pub(crate) fn role_signer(&self, role: RingRole) -> Result<SigningIdentity> {
        self.signer(self.operator_manifest()?.role(role))
    }

    /// A pin KMS does not hold would lock its role at `Initialize`.
    pub(crate) fn kms_checked_roles(&self) -> Result<RingRoles> {
        let address = |role| Ok::<_, anyhow::Error>(self.role_signer(role)?.address());
        Ok(RingRoles {
            admin: address(RingRole::Admin)?,
            policy: address(RingRole::Policy)?,
            pauser: address(RingRole::Pauser)?,
            cosign_admin: address(RingRole::CosignAdmin)?,
            enrollment_writer: address(RingRole::EnrollmentWriter)?,
        })
    }

    pub(crate) fn authority(&self) -> Result<SigningIdentity> {
        self.role_signer(RingRole::Policy)
    }

    /// For a step that only pays fees and small rent.
    pub(crate) fn funded_authority(&self) -> Result<SigningIdentity> {
        let authority = self.authority()?;
        self.fund(authority.address(), fund::MIN_AUTHORITY_BALANCE)?;
        Ok(authority)
    }

    pub(crate) fn audit_key(&self) -> Result<spr_keypair::X25519Keypair> {
        let manifest = self.operator_manifest()?;
        let wrapped = manifest
            .auditor
            .as_ref()
            .context("the operator manifest holds no wrapped ring auditor")?;
        let genesis = self.genesis()?;
        let context = SecretContext {
            deployment: &manifest.deployment,
            genesis: &genesis,
            purpose: SecretPurpose::Auditor(self.ring.program_id()),
        };
        let kms = self.kms()?;
        let secret = self.block(async { anyhow::Ok(kms.open(wrapped, context).await?) })?;
        Ok(spr_keypair::X25519Keypair::from_bytes(&secret))
    }

    /// `None` unless a localnet ring names a loopback Ring RPC.
    pub(crate) fn local_rpc_roles(&self) -> Result<Option<&RpcCustody>> {
        cached(&self.custody.local_rpc, || self.read_local_rpc_roles()).map(Option::as_ref)
    }

    fn read_local_rpc_roles(&self) -> Result<Option<RpcCustody>> {
        if self.config.target != Target::Localnet
            || !Urls::is_loopback(&self.config.urls().ring_rpc)
            || self.config.operator_manifest.is_none()
        {
            return Ok(None);
        }
        let manifest_path = self.operator_manifest_path()?;
        if self.custody.manifest.get().is_none() && !manifest_path.try_exists()? {
            return Ok(None);
        }
        self.operator_manifest()?
            .rpc_custody_path(&manifest_path)?
            .map(|path| parse_json(&path))
            .transpose()
    }

    fn kms(&self) -> Result<&Kms> {
        cached(&self.custody.kms, || {
            let endpoint = self.operator_manifest()?.kms_endpoint();
            ensure!(
                matches!(endpoint, KmsEndpoint::Aws { .. })
                    || (self.config.target == Target::Localnet
                        && Urls::is_loopback(&self.config.urls().rpc)),
                "the local KMS requires a loopback localnet RPC"
            );
            let genesis = self.genesis()?;
            self.block(async { anyhow::Ok(endpoint.connect(&genesis).await?) })
        })
    }

    fn genesis(&self) -> Result<Hash> {
        Ok(Hash::new_from_array(self.rpc().genesis_hash()?))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use solana_keypair::Keypair;

    use super::*;
    use crate::{context::ContextSetup, ui::Defaults, RingConfig};

    /// Any KMS resolution fails by name, the ring names no operator manifest.
    fn context_without_kms() -> Context {
        let local = "http://127.0.0.1:1";
        let urls = format!("rpc = \"{local}\"\nindexer = \"{local}\"\nprover = \"{local}\"\nring_rpc = \"{local}\"\n");
        let config: RingConfig = toml::from_str(&format!(
            "name = \"cache\"\ntarget = \"localnet\"\nprogram_id = \"11111111111111111111111111111111\"\n\n[localnet]\n{urls}\n[devnet]\n{urls}"
        ))
        .expect("ring.toml");
        ContextSetup {
            config_path: PathBuf::from("ring.toml"),
            config,
            ask: Box::new(Defaults),
            catalogue: None,
            prover_tee_policy: None,
            fee_payer_keypair: None,
        }
        .open()
        .expect("context")
    }

    #[test]
    fn a_resolved_signer_is_reused_only_under_the_same_address_and_key_arn() {
        let ctx = context_without_kms();
        let resolved: SigningIdentity = Arc::new(Keypair::new());
        let key_arn = |id: &str| -> KeyArn {
            format!("arn:aws:kms:us-east-1:123456789012:key/{id}")
                .parse()
                .expect("key ARN")
        };
        let pin = SignerRef {
            key_arn: key_arn("first"),
            address: resolved.address(),
        };
        ctx.custody
            .signers
            .borrow_mut()
            .insert((pin.address, pin.key_arn.clone()), resolved.clone());
        let reused = ctx.signer(&pin).expect("cached");
        assert!(Arc::ptr_eq(&reused, &resolved));
        let moved = SignerRef {
            key_arn: key_arn("second"),
            ..pin
        };
        let resolved_again = ctx.signer(&moved).err().expect("resolved again");
        assert_eq!(
            resolved_again.to_string(),
            "ring.toml names no operator_manifest, an operator command needs one"
        );
    }
}
