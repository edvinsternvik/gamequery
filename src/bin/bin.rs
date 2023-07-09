use std::time::Duration;
use gamequery::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Wrong number of arguments.\nUsage: gamequery <ip:port>");
        return;
    }

    let query_settings = ServerQuerySettings {
        ip: args[1].clone(),
        timeout: Some(Duration::from_secs(5)),
    };
    let game_info = SteamServerInfo::query(query_settings).expect("Could not query game info");

    println!("{:?}", game_info);
}
