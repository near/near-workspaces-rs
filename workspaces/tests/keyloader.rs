use workspaces::{types::keyloader::KeyLoader, Account};

const NODE_NET: &str = "sandbox";

#[tokio::test]
async fn test_keyloader() -> anyhow::Result<()> {
    // creating an account and saving credentials to keychain
    let worker = workspaces::sandbox().await?;
    let (id, sk) = worker.dev_generate().await;
    let res = worker.create_tla(id.clone(), sk.clone()).await?;
    assert!(res.is_success());

    let credentials = KeyLoader::new(sk.clone(), sk.public_key());
    credentials.to_keychain(NODE_NET, &id).await?;

    // retrieve from keychain, view account
    let account = KeyLoader::from_keychain(&worker, NODE_NET, id.clone()).await?;
    let res = Account::from_secret_key(id, account.private_key.into(), &worker)
        .view_account()
        .await?;

    assert!(res.balance > 0);

    Ok(())
}
