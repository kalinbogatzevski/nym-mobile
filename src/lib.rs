use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use nym_client::persistence::pathfinder::Pathfinder;
use nym_client::persistence::pemstore;
use nym_client::persistence::pemstore::PemStore;
use nym_client::clients::{NymClient, SocketType};
use clap::ArgMatches;
use crypto::identity::{MixnetIdentityKeyPair, MixnetIdentityPublicKey};
use std::net::{ToSocketAddrs, SocketAddr};
use std::path::Path;

#[no_mangle]
pub unsafe extern "C" fn init(id: *const c_char) -> *mut c_char {
    let c_str = CStr::from_ptr(id);
    let recipient = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => "you",
    };
    let pathfinder = Pathfinder::new(recipient.to_string());

    if Path::new(&pathfinder.config_dir).exists() {
        return CString::new(format!("The id already exists."))
            .unwrap()
            .into_raw()
    }

    println!("Writing keypairs to {:?}...", pathfinder.config_dir);
    let mix_keys = crypto::identity::DummyMixIdentityKeyPair::new();
    let pem_store = PemStore::new(pathfinder);
    pem_store.write_identity(mix_keys);

    CString::new(format!("Hello"))
        .unwrap()
        .into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn start_ws(id: *const c_char, directory: *const c_char) {
    let c_str = CStr::from_ptr(id);
    let recipient = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => "you",
    };

    let directory_str = CStr::from_ptr(directory);
    let directory_endpoint = match directory_str.to_str() {
        Ok(s) => s,
        Err(_) => "https://directory.nymtech.net",
    };

    let directory_server = directory_endpoint.to_string();
    println!("Listening for messages...");

    let socket_address = "127.0.0.1:9001"
        .to_socket_addrs()
        .expect("Failed to combine host and port")
        .next()
        .expect("Failed to extract the socket address from the iterator");

    let keypair = pemstore::read_mix_identity_keypair_from_disk(recipient.to_string());
    // TODO: reading auth_token from disk (if exists);

    let mut temporary_address = [0u8; 32];
    let public_key_bytes = keypair.public_key().to_bytes();
    temporary_address.copy_from_slice(&public_key_bytes[..]);
    let auth_token = None;
    let client = NymClient::new(
        temporary_address,
        socket_address,
        directory_server,
        auth_token,
        SocketType::WebSocket,
    );

    client.start().unwrap();

    
}

#[no_mangle]
pub unsafe extern "C" fn init_release(to: *mut c_char) {
    if to.is_null() {
        return;
    }
    CString::from_raw(to);
}

#[no_mangle]
pub unsafe extern "C" fn start_ws_release(to: *mut c_char) {
    if to.is_null() {
        return;
    }
    CString::from_raw(to);
}