use std::str::FromStr;

use near_primitives::borsh::{self, BorshDeserialize};

use near_workspaces::AccountId;
use near_workspaces::types::{KeyType, PublicKey, SecretKey};

fn default_workspaces_pubkey() -> anyhow::Result<PublicKey> {
    let data = bs58::decode("279Zpep9MBBg4nKsVmTQE7NbXZkWdxti6HS1yzhp8qnc1ExS7gU").into_vec()?;
    Ok(PublicKey::try_from_slice(data.as_slice())?)
}

#[test]
fn test_keypair_ed25519() -> anyhow::Result<()> {
    let pk_expected = "\"ed25519:DcA2MzgpJbrUATQLLceocVckhhAqrkingax4oJ9kZ847\"";
    let sk_expected = "\"ed25519:3KyUuch8pYP47krBq4DosFEVBMR5wDTMQ8AThzM8kAEcBQEpsPdYTZ2FPX5ZnSoLrerjwg66hwwJaW1wHzprd5k3\"";

    let sk = SecretKey::from_seed(KeyType::ED25519, "test");
    let pk = sk.public_key();
    assert_eq!(serde_json::to_string(&pk)?, pk_expected);
    assert_eq!(serde_json::to_string(&sk)?, sk_expected);
    assert_eq!(pk, serde_json::from_str(pk_expected)?);
    assert_eq!(sk, serde_json::from_str(sk_expected)?);

    Ok(())
}

#[test]
fn test_keypair_secp256k1() -> anyhow::Result<()> {
    let pk_expected = "\"secp256k1:5ftgm7wYK5gtVqq1kxMGy7gSudkrfYCbpsjL6sH1nwx2oj5NR2JktohjzB6fbEhhRERQpiwJcpwnQjxtoX3GS3cQ\"";
    let sk_expected = "\"secp256k1:X4ETFKtQkSGVoZEnkn7bZ3LyajJaK2b3eweXaKmynGx\"";

    let sk = SecretKey::from_seed(KeyType::SECP256K1, "test");
    let pk = sk.public_key();
    assert_eq!(serde_json::to_string(&pk)?, pk_expected);
    assert_eq!(serde_json::to_string(&sk)?, sk_expected);
    assert_eq!(pk, serde_json::from_str(pk_expected)?);
    assert_eq!(sk, serde_json::from_str(sk_expected)?);

    Ok(())
}

#[test]
fn test_keypair_mldsa65() -> anyhow::Result<()> {
    // ML-DSA-65 keygen and string round-tripping, without a sandbox.
    for sk in [
        SecretKey::from_random(KeyType::MLDSA65),
        SecretKey::from_seed(KeyType::MLDSA65, "test"),
    ] {
        assert!(matches!(sk.key_type(), KeyType::MLDSA65));

        let pk = sk.public_key();
        assert!(matches!(pk.key_type(), KeyType::MLDSA65));
        // Full ML-DSA-65 public key is 1952 bytes of key data (1953 with the
        // leading key-type byte).
        assert_eq!(pk.key_data().len(), 1952);
        assert_eq!(pk.len(), 1953);
        assert_eq!(KeyType::MLDSA65.data_len(), 1952);

        // Both keys round-trip through their `ml-dsa-65:`-prefixed string form.
        let pk_str = pk.to_string();
        assert!(pk_str.starts_with("ml-dsa-65:"), "got {pk_str}");
        assert_eq!(PublicKey::from_str(&pk_str)?, pk);

        let sk_str = sk.to_string();
        assert!(sk_str.starts_with("ml-dsa-65:"), "got {sk_str}");
        assert_eq!(SecretKey::from_str(&sk_str)?, sk);
    }

    // The numeric and string discriminants both resolve to ML-DSA-65.
    assert!(matches!(KeyType::try_from(2u8)?, KeyType::MLDSA65));
    assert!(matches!(KeyType::from_str("ml-dsa-65")?, KeyType::MLDSA65));

    Ok(())
}

#[test]
fn test_pubkey_serialization() -> anyhow::Result<()> {
    for key_type in [KeyType::ED25519, KeyType::SECP256K1] {
        let sk = SecretKey::from_seed(key_type, "test");
        let pk = sk.public_key();
        let bytes = borsh::to_vec(&pk)?;

        // Borsh Deserialization should equate to the original public key:
        assert_eq!(PublicKey::try_from_slice(&bytes)?, pk);

        // invalid public key should error out on deserialization:
        assert!(PublicKey::try_from_slice(&[0]).is_err());
    }

    Ok(())
}

#[cfg(feature = "interop_sdk")]
#[tokio::test]
async fn test_pubkey_from_sdk_ser() -> anyhow::Result<()> {
    const TYPE_SER_BYTES: &[u8] =
        include_bytes!("test-contracts/type-serialize/res/test_contract_type_serialization.wasm");
    let worker = near_workspaces::sandbox().await?;
    let contract = worker.dev_deploy(TYPE_SER_BYTES).await?;

    // Test out serde serialization and deserialization for PublicKey
    let ws_pk = default_workspaces_pubkey()?;
    let sdk_pk: PublicKey = contract
        .call("pass_pk_back_and_forth")
        .args_json(serde_json::json!({ "pk": ws_pk }))
        .transact()
        .await?
        .json()?;
    assert_eq!(ws_pk, sdk_pk);

    // Test out borsh serialization and deserialization for PublicKey
    let sdk_pk: PublicKey = contract
        .call("pass_borsh_pk_back_and_forth")
        .args_borsh(&ws_pk)
        .transact()
        .await?
        .borsh()?;
    assert_eq!(ws_pk, sdk_pk);

    Ok(())
}

#[test]
fn test_pubkey_borsh_format_change() -> anyhow::Result<()> {
    let pk = default_workspaces_pubkey()?;
    assert_eq!(
        borsh::to_vec(&pk)?,
        bs58::decode("279Zpep9MBBg4nKsVmTQE7NbXZkWdxti6HS1yzhp8qnc1ExS7gU").into_vec()?
    );

    Ok(())
}

#[test]
fn test_valid_account_id() {
    let account_id = "testnet";
    assert!(
        AccountId::from_str(account_id).is_ok(),
        "Something changed underneath for testnet to not be a valid Account ID"
    );
}
