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

    let shutdown_stream = client.on_steam_shutdown();

    let task = smol::spawn(async move {
        let _ = shutdown_stream.await;
        assert!(true);
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

    let shutdown_stream = client.on_steam_shutdown();
    let another_shutdown_stream = client.on_steam_shutdown();

    let task = smol::spawn(async move {
        assert!(shutdown_stream.await.is_err());
    });

    let another_task = smol::spawn(async move {
        assert!(another_shutdown_stream.await.is_ok());
    });

    smol::block_on(async move {
        client.shutdown();
        task.await;
        another_task.await;
    });
}
