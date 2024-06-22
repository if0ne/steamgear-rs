use steamgear::api::SteamApi;

fn main() {
    let steam_api = SteamApi::new_client(None).unwrap();

    dbg!(steam_api.apps().get_launch_command_line());

    steam_api.shutdown();
}
