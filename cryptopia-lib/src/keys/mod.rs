pub use cryptopia_core::format::keys::*;
use cryptopia_core::format::FormatError;
pub use cryptopia_core::hybrid_kem::{DHAlgorithm, DHKeyPair, KEMAlgorithm, KEMKeyPair};
pub use cryptopia_core::hybrid_sign::{ECAlgorithm, ECKeyPair, PQAlgorithm, PQKeyPair};
pub use cryptopia_core::seed::Seed;
use secrecy::{ExposeSecret, SecretVec};
use std::io::{BufReader, Read, Write};

pub struct HybridSignAlgorithm {
    ec_algorithm: ECAlgorithm,
    pq_algorithm: PQAlgorithm,
}

pub struct HybridKEMAlgorithm {
    dh_algorithm: DHAlgorithm,
    kem_algorithm: KEMAlgorithm,
}

pub struct MasterKey {
    hybrid_kem_algorithm: HybridKEMAlgorithm,
    hybrid_sign_algorithm: HybridSignAlgorithm,
    master_seed: Seed,
}

impl MasterKey {
    pub fn generate(
        hybrid_kem_algorithm: HybridKEMAlgorithm,
        hybrid_sign_algorithm: HybridSignAlgorithm,
    ) -> Self {
        let master_seed = Seed::generate();

        MasterKey {
            hybrid_kem_algorithm,
            hybrid_sign_algorithm,
            master_seed: master_seed,
        }
    }

    pub fn export<W: Write>(&self, passphrase_bytes: Option<SecretVec<u8>>, mut output: W) {
        //TODO: Encrypt with cryptopia_core::encryption if passphrase is provided

        match passphrase_bytes {
            Some(passphrase) => {
                todo!()
            }
            None => {
                // TODO: impl `From` in cryptopia_core::format::keys
                let secret_key_format = SecretKeyFormat {
                    ec_algorithm: self.hybrid_sign_algorithm.ec_algorithm,
                    pq_algorithm: self.hybrid_sign_algorithm.pq_algorithm,
                    dh_algorithm: self.hybrid_kem_algorithm.dh_algorithm,
                    kem_algorithm: self.hybrid_kem_algorithm.kem_algorithm,
                    master_seed: self.master_seed.clone_raw_seed(),
                    encryption_metadata: None,
                };

                output.write_all(secret_key_format.encode().expose_secret());
            }
        }
    }

    pub fn is_encrypted(secret_key_format: &SecretKeyFormat) -> bool {
        secret_key_format.encryption_metadata.is_some()
    }

    pub fn decode_bson<R: Read>(serialized_master_key: R) -> Result<SecretKeyFormat, FormatError> {
        let reader = BufReader::new(serialized_master_key);
        SecretKeyFormat::decode(&SecretVec::from(reader.buffer().to_vec()))
    }

    pub fn import(
        decoded_master_key_format: SecretKeyFormat,
        passphrase_bytes: Option<SecretVec<u8>>,
    ) -> Result<Self, FormatError> {
        let hybrid_sign_algorithm = HybridSignAlgorithm {
            ec_algorithm: decoded_master_key_format.ec_algorithm,
            pq_algorithm: decoded_master_key_format.pq_algorithm,
        };

        let hybrid_kem_algorithm = HybridKEMAlgorithm {
            dh_algorithm: decoded_master_key_format.dh_algorithm,
            kem_algorithm: decoded_master_key_format.kem_algorithm,
        };

        match passphrase_bytes {
            Some(passphrase) => {
                todo!()
            }
            None => Ok(MasterKey {
                hybrid_sign_algorithm,
                hybrid_kem_algorithm,
                master_seed: Seed::new(decoded_master_key_format.master_seed),
            }),
        }
    }

    pub fn get_signing_keypair(&self) -> (ECKeyPair, PQKeyPair) {
        let ec_algorithm = self.hybrid_sign_algorithm.ec_algorithm;
        let pq_algorithm = self.hybrid_sign_algorithm.pq_algorithm;

        let ec_keypair = ECKeyPair::from_seed(&self.master_seed, ec_algorithm);
        let pq_keypair = PQKeyPair::from_seed(&self.master_seed, pq_algorithm);

        (ec_keypair, pq_keypair)
    }

    pub fn get_encryption_keypair(&self) -> (DHKeyPair, KEMKeyPair) {
        let dh_algorithm = self.hybrid_kem_algorithm.dh_algorithm;
        let kem_algorithm = self.hybrid_kem_algorithm.kem_algorithm;

        let dh_keypair = DHKeyPair::from_seed(&self.master_seed, dh_algorithm);
        let kem_keypair = KEMKeyPair::from_seed(&self.master_seed, kem_algorithm);

        (dh_keypair, kem_keypair)
    }

    pub fn get_signing_public_key(&self) -> SignaturePublicKeyFormat {
        let (ec_keypair, pq_keypair) = self.get_signing_keypair();

        SignaturePublicKeyFormat::from_keypairs(ec_keypair, pq_keypair)
    }

    pub fn get_encryption_public_key(&self) -> EncryptionPublicKeyFormat {
        let (dh_keypair, kem_keypair) = self.get_encryption_keypair();

        EncryptionPublicKeyFormat::from_keypairs(dh_keypair, kem_keypair)
    }

    pub fn export_public<W: Write>(&self, mut output: W) {
        let signature_public_key = self.get_signing_public_key();
        let encryption_public_key = self.get_encryption_public_key();

        let fullchain_public_key = FullChainPublicKeyFormat {
            signature_public_key,
            encryption_public_key,
        };

        output.write_all(&fullchain_public_key.encode());
    }
}
