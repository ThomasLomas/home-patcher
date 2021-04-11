extern crate yaml_rust;
extern crate clap;
extern crate linked_hash_map;
extern crate futures;
extern crate tokio;
extern crate thrussh;
extern crate thrussh_keys;
use clap::{Arg, App};
use yaml_rust::{Yaml, YamlLoader};
use std::fs;
use std::sync::Arc;
use thrussh::*;
use thrussh_keys::*;

struct Client {
}

impl client::Handler for Client {
   type Error = anyhow::Error;
   type FutureUnit = futures::future::Ready<Result<(Self, client::Session), anyhow::Error>>;
   type FutureBool = futures::future::Ready<Result<(Self, bool), anyhow::Error>>;

   fn finished_bool(self, b: bool) -> Self::FutureBool {
        futures::future::ready(Ok((self, b)))
   }
   fn finished(self, session: client::Session) -> Self::FutureUnit {
        futures::future::ready(Ok((self, session)))
   }
   fn check_server_key(self, server_public_key: &key::PublicKey) -> Self::FutureBool {
        println!("check_server_key: {:?}", server_public_key);
        self.finished_bool(true)
   }
   fn channel_open_confirmation(self, channel: ChannelId, _max_packet_size: u32, _window_size: u32, session: client::Session) -> Self::FutureUnit {
        println!("channel_open_confirmation: {:?}", channel);
        self.finished(session)
   }
   fn data(self, channel: ChannelId, data: &[u8], session: client::Session) -> Self::FutureUnit {
        println!("data on channel {:?}: {:?}", channel, std::str::from_utf8(data));
        self.finished(session)
   }
}

fn get_config(config_path: &str) -> linked_hash_map::LinkedHashMap<Yaml, Yaml> {
    let config_contents = fs::read_to_string(config_path).expect("Unable to open config file");
    let config = YamlLoader::load_from_str(&config_contents).unwrap();
    return config[0].as_hash().unwrap().clone();
}

#[tokio::main]
async fn main() {
    let matches = App::new("Home Patcher")
        .version("1.0")
        .author("Thomas Lomas")
        .about("Used for checking on installed packages and remotely updating")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file")
                .takes_value(true)
        )
        .get_matches();

    let config = get_config(matches.value_of("config").unwrap_or("config.yaml"));
    let hosts = &config[&Yaml::String("hosts".to_owned())].as_vec();

    for host in hosts.unwrap() {
        let hostname = host["hostname"].as_str().unwrap();
        let username = host["username"].as_str().unwrap();
        let password = host["password"].as_str().unwrap();

        println!("--------------");
        println!("Hostname: {:?} || Username: {:?}", hostname, username);
        println!("Connecting...");

        let config = thrussh::client::Config::default();
        let config = Arc::new(config);
        let sh = Client{};
        let mut session = thrussh::client::connect(config, hostname, sh).await.unwrap();

        if session.authenticate_password(username, password).await.unwrap() {
            println!("Connected!");
        } else {
            println!("Could not connect");
        }
    }
}
