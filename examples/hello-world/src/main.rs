use steamgear::SteamApi;

fn main() {
    let steam_api = SteamApi::new_client(None).unwrap();

    steam_api.shutdown();
}
