use steamgear::prelude::*;

#[test]
fn steam_api_init() {
    let client = SteamClient::new_client();

    assert!(client.is_ok());
}
