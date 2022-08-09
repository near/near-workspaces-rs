use std::str::FromStr;

use workspaces::types::{KeyType, SecretKey};
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
    let pk_expected = "\"secp256k1:BtJtBjukUQbcipnS78adSwUKE38sdHnk7pTNZH7miGXfodzUunaAcvY43y37nm7AKbcTQycvdgUzFNWsd7dgPZZ\"";
    let sk_expected = "\"secp256k1:9ZNzLxNff6ohoFFGkbfMBAFpZgD7EPoWeiuTpPAeeMRV\"";

    let sk = SecretKey::from_seed(KeyType::SECP256K1, "test");
    let pk = sk.public_key();
    assert_eq!(serde_json::to_string(&pk)?, pk_expected);
    assert_eq!(serde_json::to_string(&sk)?, sk_expected);
    assert_eq!(pk, serde_json::from_str(pk_expected)?);
    assert_eq!(sk, serde_json::from_str(sk_expected)?);

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
