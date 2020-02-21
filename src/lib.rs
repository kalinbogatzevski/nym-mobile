
use nym_client::client::NymClient;
use nym_client::config::Config;
use std::ffi::{CStr};
use std::os::raw::c_char;
use nym_client::config::persistance::pathfinder::ClientPathfinder;
use pemstore::pemstore::PemStore;
use crypto::identity::MixIdentityKeyPair;
use nym_client::built_info;
use directory_client::presence::Topology;
use sphinx::route::DestinationAddressBytes;
use sfw_provider_requests::AuthToken;
use topology::provider::Node;
use topology::NymTopology;
use nym_client::config::SocketType;
use config::NymConfig;

use env_logger::Env;

#[no_mangle]
pub unsafe extern "C" fn init(id: *const c_char, _directory: *const c_char) {
    let c_str = CStr::from_ptr(id);
    let _recipient = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => "you",
    };

    let directory_str = CStr::from_ptr(_directory);
    let directory_endpoint = match directory_str.to_str() {
        Ok(s) => s,
        Err(_) => "https://directory.nymtech.net",
    };

    // let pathfinder = ClientPathfinder::new(recipient.to_string());
    // if Path::new(&pathfinder.config_dir).exists() {
    //     return CString::new(format!("The id already exists."))
    //         .unwrap()
    //         .into_raw()
    // }
    // println!("Writing keypairs to {:?}...", pathfinder.config_dir);
   
    let mut config = nym_client::config::Config::new(_recipient);
    config = config.with_custom_directory(directory_endpoint)
        .with_socket(SocketType::WebSocket)
        .with_port(1707);

    let mix_identity_keys = MixIdentityKeyPair::new();

    // if there is no provider chosen, get a random-ish one from the topology
    if config.get_provider_id().is_empty() {
        let our_address = mix_identity_keys.public_key().derive_address();
        // TODO: is there perhaps a way to make it work without having to spawn entire runtime?
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        let (provider_id, auth_token) =
            rt.block_on(choose_provider(config.get_directory_server(), our_address));
        config = config
            .with_provider_id(provider_id)
            .with_provider_auth_token(auth_token);
    }

    let pathfinder = ClientPathfinder::new_from_config(&config);
    let pem_store = PemStore::new(pathfinder);
    pem_store
        .write_identity(mix_identity_keys)
        .expect("Failed to save identity keys");
    println!("Saved mixnet identity keypair");

    let config_save_location = config.get_config_file_save_location();
    config
        .save_to_file(None)
        .expect("Failed to save the config file");
    println!("Saved configuration file to {:?}", config_save_location);

    println!(
        "Unless overridden in all `nym-client run` we will be talking to the following provider: {}...",
        config.get_provider_id(),
    );
    if config.get_provider_auth_token().is_some() {
        println!(
            "using optional AuthToken: {:?}",
            config.get_provider_auth_token().unwrap()
        )
    }
    println!("Client configuration completed.\n\n\n");
}

#[no_mangle]
pub unsafe extern "C" fn start_ws(id: *const c_char, _directory: *const c_char) {
    let c_str = CStr::from_ptr(id);
    let _recipient = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => "you",
    };
    
    env_logger::from_env(Env::default().default_filter_or("debug")).init();

    let pathfinder = ClientPathfinder::new(_recipient.to_string());
    let config = Config::load_from_file(Some(pathfinder.config_dir), Some(_recipient))
            .expect("Failed to load config file");

    let client = NymClient::new(config);
    client.start().unwrap();
}

// in the long run this will be provider specific and only really applicable to a
// relatively small subset of all providers
async fn choose_provider(
    directory_server: String,
    our_address: DestinationAddressBytes,
) -> (String, AuthToken) {
    // TODO: once we change to graph topology this here will need to be updated!
    let topology = Topology::new(directory_server.clone());
    let version_filtered_topology = topology.filter_node_versions(
        built_info::PKG_VERSION,
        built_info::PKG_VERSION,
        built_info::PKG_VERSION,
    );
    // don't care about health of the networks as mixes can go up and down any time,
    // but DO care about providers
    let providers = version_filtered_topology.providers();

    // try to perform registration so that we wouldn't need to do it at startup
    // + at the same time we'll know if we can actually talk with that provider
    let registration_result = try_provider_registrations(providers, our_address).await;
    match registration_result {
        None => {
            // while technically there's no issue client-side, it will be impossible to execute
            // `nym-client run` as no provider is available so it might be best to not finalize
            // the init and rely on users trying to init another time?
            panic!(
                "Currently there are no valid providers available on the network ({}). \
                 Please try to run `init` again at later time or change your directory server",
                directory_server
            )
        }
        Some((provider_id, auth_token)) => (provider_id, auth_token),
    }
}

async fn try_provider_registrations(
    providers: Vec<Node>,
    our_address: DestinationAddressBytes,
) -> Option<(String, AuthToken)> {
    // since the order of providers is non-deterministic we can just try to get a first 'working' provider
    for provider in providers {
        let provider_client =
            provider_client::ProviderClient::new(provider.client_listener, our_address, None);
        let auth_token = provider_client.register().await;
        if let Ok(token) = auth_token {
            return Some((provider.pub_key, token));
        }
    }
    None
}