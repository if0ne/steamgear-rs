use futures::StreamExt;
use steamgear::prelude::*;

#[test]
fn steam_api_init() {
    let client = SteamClient::new_client();
    assert!(client.is_ok());
    let client = client.unwrap();

    client.shutdown();
}

#[test]
fn steam_api_callback_shutdown() {
    let client = SteamClient::new_client();
    assert!(client.is_ok());
    let client = client.unwrap();

    let mut shutdown_stream = client.on_steam_shutdown();

    let task = smol::spawn(async move {
        assert!(shutdown_stream.next().await.is_some());
    });

    smol::block_on(async move {
        client.shutdown();
        task.await;
    });
}

#[test]
fn steam_api_callback_override() {
    let client = SteamClient::new_client();
    assert!(client.is_ok());
    let client = client.unwrap();

    let mut shutdown_stream = client.on_steam_shutdown();
    let mut another_shutdown_stream = client.on_steam_shutdown();

    let task = smol::spawn(async move {
        assert!(shutdown_stream.next().await.is_none());
    });

    let another_task = smol::spawn(async move {
        assert!(another_shutdown_stream.next().await.is_some());
    });

    smol::block_on(async move {
        client.shutdown();
        task.await;
        another_task.await;
    });
}
