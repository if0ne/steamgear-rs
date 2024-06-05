use std::io::Write;

fn main() {
    println!("cargo::rerun-if-changed=steam_appid.txt");
    if std::fs::File::open("steam_appid.txt").is_err() {
        let buffer = "480".as_bytes();
        let mut file =
            std::fs::File::create("steam_appid.txt").expect("Couldn't create steam_appid.txt!");

        file.write_all(buffer)
            .expect("Couldn't write steam app id!");
    }
}
