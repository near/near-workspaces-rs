use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};

use workspaces::types::{KeyType, PublicKey, SecretKey};
use workspaces::AccountId;

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

    println!("{}", pk.len());
    println!("{}", pk.key_data().len());
    let st = format!("{}", pk);
    println!("{}", st.len());
    assert_eq!(serde_json::to_string(&pk)?, pk_expected);
    assert_eq!(serde_json::to_string(&sk)?, sk_expected);
    assert_eq!(pk, serde_json::from_str(pk_expected)?);
    assert_eq!(sk, serde_json::from_str(sk_expected)?);

    Ok(())
}

#[test]
fn test_borsh_on_pubkey() -> anyhow::Result<()> {
    for key_type in [KeyType::ED25519, KeyType::SECP256K1] {
        let sk = SecretKey::from_seed(key_type, "test");
        let pk = sk.public_key();
        let bytes = pk.try_to_vec()?;

        // Deserialization should equate to the original public key:
        assert_eq!(PublicKey::try_from_slice(&bytes)?, pk);

        // invalid public key should error out on deserialization:
        assert!(PublicKey::try_from_slice(&[0]).is_err());
    }

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
