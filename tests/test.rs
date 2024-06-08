use smol::stream::StreamExt;
use steamgear::prelude::*;

#[test]
fn steam_api_init() {
    let client = SteamClient::new_client();

    assert!(client.is_ok());
}

#[test]
fn steam_api_callback_shutdown() {
    let client = SteamClient::new_client();

    assert!(client.is_ok());

    let client = client.unwrap();

    let shutdown_stream = client.on_steam_shutdown();
    
    let task = smol::spawn(async move {
        let mut shutdown_result = std::pin::pin!(shutdown_stream);
        assert!(shutdown_result.next().await.is_some());
    });

    smol::block_on(async move {
        client.shutdown().await;
        task.await;
    });
}