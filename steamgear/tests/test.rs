#[cfg(test)]
mod tests {
    use smol::stream::StreamExt;
    use steamgear::api::SteamApi;

    #[test]
    fn steam_api_init() {
        let client = SteamApi::new_client(None);
        assert!(client.is_ok());
        let client = client.unwrap();

        client.shutdown();
    }

    #[test]
    fn steam_api_callback_shutdown() {
        let client = SteamApi::new_client(None);
        assert!(client.is_ok());
        let client = client.unwrap();

        let shutdown_stream = client.utils().on_steam_shutdown();

        let task = smol::spawn(async move {
            let mut shutdown_stream = std::pin::pin!(shutdown_stream);
            assert!(shutdown_stream.next().await.is_some());
        });

        smol::block_on(async move {
            client.shutdown();
            task.await;
        });
    }

    #[test]
    fn steam_api_callback_override() {
        let client = SteamApi::new_client(None);
        assert!(client.is_ok());
        let client = client.unwrap();

        let shutdown_stream = client.utils().on_steam_shutdown();
        let another_shutdown_stream = client.utils().on_steam_shutdown();

        let task = smol::spawn(async move {
            let mut shutdown_stream = std::pin::pin!(shutdown_stream);
            assert!(shutdown_stream.next().await.is_none());
        });

        let another_task = smol::spawn(async move {
            let mut another_shutdown_stream = std::pin::pin!(another_shutdown_stream);
            assert!(another_shutdown_stream.next().await.is_some());
        });

        smol::block_on(async move {
            client.shutdown();
            task.await;
            another_task.await;
        });
    }
}
