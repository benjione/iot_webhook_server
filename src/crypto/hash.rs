extern crate blake2;
extern crate sha3;

use blake2::digest::Digest;
use sha3::Sha3_256;
use std::str;

fn hash_password(password: &String, salt: &str) -> String {
    let mut hasher = Sha3_256::new();
    hasher.input(password.as_bytes());
    hasher.input(b"$");
    hasher.input(salt.as_bytes());
    let output: String = format!("{:x}", hasher.result());
    return output;
}


pub fn hash_sha256(hash_me: &String) -> String {
    return hash_password(hash_me, "abc123");
}
