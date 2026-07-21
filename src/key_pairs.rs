use base64::Engine;
use base64::engine::general_purpose;
use crypto_box::SecretKey;
use ed25519_dalek::SigningKey;
use ed25519_dalek::pkcs8::EncodePublicKey;
use secrecy::{ExposeSecret, SecretBox};
use spki::der::asn1::BitString;
use spki::der::pem::LineEnding;
use spki::der::zeroize::Zeroize;
use spki::der::{Any, EncodePem};
use spki::{AlgorithmIdentifier, ObjectIdentifier, SubjectPublicKeyInfo};

#[derive(Clone)]
pub struct ApplicationKeyPairs {
    pub x25519_keypair: X25519KeyPair,
    pub ed25519_keypair: Ed25519KeyPair,
}

impl ApplicationKeyPairs {
    pub fn build(secret_key_seed: &SecretBox<str>) -> Self {
        let mut secret_key_seed_bytes = Self::get_seed_bytes(secret_key_seed);

        let x25519_keypair = X25519KeyPair::build(secret_key_seed_bytes);
        let ed25519_keypair = Ed25519KeyPair::build(secret_key_seed_bytes);

        secret_key_seed_bytes.zeroize();

        Self {
            x25519_keypair,
            ed25519_keypair,
        }
    }

    fn get_seed_bytes(secret_key_seed: &SecretBox<str>) -> [u8; 32] {
        let secret_key_seed = secret_key_seed.expose_secret();
        let mut decoded_secret_key_seed = general_purpose::STANDARD
            .decode(secret_key_seed.trim())
            .expect("Failed to decode secret key");

        if decoded_secret_key_seed.len() != 32 {
            panic!("secret key seed must be 32 bytes long")
        }

        let mut secret_key_seed_bytes = [0u8; 32];
        secret_key_seed_bytes.copy_from_slice(&decoded_secret_key_seed);
        decoded_secret_key_seed.zeroize();

        secret_key_seed_bytes
    }
}

#[derive(Clone)]
pub struct X25519KeyPair {
    pub secret_key: SecretKey,
    pub public_key_pem: String,
}

impl X25519KeyPair {
    pub fn build(secret_key_seed_bytes: [u8; 32]) -> Self {
        let secret_key = SecretKey::from_bytes(secret_key_seed_bytes);

        let public_key = secret_key.public_key();

        let bit_string = BitString::new(0, public_key.as_bytes())
            .expect("Failed to create ASN.1 BIT STRING from a byte slice.");

        let spki = SubjectPublicKeyInfo::<Any, BitString> {
            algorithm: AlgorithmIdentifier {
                oid: ObjectIdentifier::new("1.3.101.110").expect("Failed to create OID."),
                parameters: None,
            },
            subject_public_key: bit_string,
        };

        let public_key_pem = spki
            .to_pem(LineEnding::LF)
            .expect("Failed to generate pem.");

        Self {
            secret_key,
            public_key_pem,
        }
    }
}

#[derive(Clone)]
pub struct Ed25519KeyPair {
    pub signing_key: SigningKey,
    pub verifying_key_pem: String,
}

impl Ed25519KeyPair {
    pub fn build(secret_key_seed_bytes: [u8; 32]) -> Self {
        let signing_key = SigningKey::from_bytes(&secret_key_seed_bytes);
        let verifying_key = signing_key.verifying_key();

        let verifying_key_pem = verifying_key
            .to_public_key_pem(LineEnding::LF)
            .expect("Failed to parse public key pem.");

        Self {
            signing_key,
            verifying_key_pem,
        }
    }
}
