// use futures::prelude::*;
use clap::{load_yaml, App};
use config::Node;
use std::error::Error;
use types::{Block, ProtocolMsg, Transaction};

fn main() -> Result<(), Box<dyn Error>> {
    let yaml = load_yaml!("cli.yml");
    let m = App::from_yaml(yaml).get_matches();

    let conf_str = m
        .value_of("config")
        .expect("unable to convert config file into a string");
    let conf_file = std::path::Path::new(conf_str);
    let str = String::from(conf_str);
    let mut config = match conf_file
        .extension()
        .expect("Unable to get file extension")
        .to_str()
        .expect("Failed to convert the extension into ascii string")
    {
        "json" => Node::from_json(str),
        "dat" => Node::from_bin(str),
        "toml" => Node::from_toml(str),
        "yaml" => Node::from_yaml(str),
        _ => panic!("Invalid config file extension"),
    };
    config.validate().expect("The decoded config is not valid");
    if let Some(d) = m.value_of("delta") {
        config.delta = d.parse().unwrap();
    }
    if let Some(f) = m.value_of("ip") {
        config.update_config(util::io::file_to_ips(f.to_string()));
    }
    println!("{:?}", config.net_map);
    let config = config;
    let mut is_client_apollo_enabled = false;
    if let Some(_x) = m.value_of("special_client") {
        is_client_apollo_enabled = true;
    }

    simple_logger::SimpleLogger::new().init().unwrap();
    let x = m.occurrences_of("debug");
    match x {
        0 => log::set_max_level(log::LevelFilter::Info),
        1 => log::set_max_level(log::LevelFilter::Debug),
        2 | _ => log::set_max_level(log::LevelFilter::Trace),
    }
    unsafe {
        config_lc::SLEEP_TIME = 10 + 4 * config.num_nodes as u64;
    }

    log::info!(target:"app","Successfully decoded the config file");

    let cli_net_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    // Setup client network
    let (cli_send, cli_recv) =
        cli_net_rt.block_on(net::Protocol::<Transaction, Block>::client_setup(
            config.client_ip(),
            util::codec::EnCodec::new(),
            util::codec::tx::Codec::new(),
        ));

    let prot_net_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    // Setup networking
    let protocol_network =
        net::Protocol::<ProtocolMsg, ProtocolMsg>::new(config.id, config.num_nodes as u16);

    // Setup the protocol network
    let (net_send, net_recv) = prot_net_rt.block_on(protocol_network.server_setup(
        config.net_map.clone(),
        util::codec::EnCodec::new(),
        util::codec::proto::Codec::new(),
    ));

    let core_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .unwrap();

    // Start the Apollo consensus protocol
    core_rt.block_on(consensus::bft::node::reactor(
        &config,
        is_client_apollo_enabled,
        net_send,
        net_recv,
        cli_send,
        cli_recv,
    ));
    Ok(())
}
