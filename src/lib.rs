use std::io::{stdout, Write};
//use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use std::str;
use std::error::Error;
use std::fs;
//use std::fs::OpenOptions;
//use std::io::{BufWriter};
use std::collections::HashMap;
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use rand::RngCore;
use rand::rngs::OsRng;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use sha2::{Digest, Sha256};

use crossterm:: {
    cursor,
    terminal,
    execute,
};

pub mod structs;

use crate::structs::Credential;

//SiteUser is a Hashmap with key as a "user" of the site, and Credential 
//as the value. This allows CredentialStore to store credentials of 
//multiple users on the same website

pub type SiteUser = HashMap<String, Credential>;

//Credential Store is a hashmap keyed by "site", and stores SiteUser as value
pub type CredentialStore = HashMap<String, SiteUser>;

// Derives a 32-byte master key from the provided master password using SHA-256.
//
// # Arguments
//
// * `master_password` - The user-supplied master password as a string slice.
//
// # Returns "Result" of:
//
// OK(A 32-byte array suitable for use as an AES-256-GCM encryption key)
// Error("Invalid Master Password")
//
pub fn derive_master_key(master_password:&str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(master_password.as_bytes());
    let master_key = hasher.finalize().into();
    //println!("derive_master_key: Returning Master key: {:?}", master_key);
    master_key
}


/// Verifies the master password against the stored master key hash.
pub fn verify_master_password(
        master_password: &str, 
        master_key_hash: &[u8]) -> 
        Result<[u8; 32], String> {
    
    let master_key = derive_master_key(master_password);
    //if master_key == MASTER_KEY_HASH {
    if master_key == master_key_hash {
        Ok(master_key)
    }
    else {
        Err("Invalid Master Password".to_string())
    }
}

/// Generates a random 12-byte nonce for AES-GCM encryption
pub fn generate_nonce() -> [u8; 12] {
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    nonce_bytes
}

/// Encrypts data using AES-256-GCM
/// Returns a vector containing: [nonce (12 bytes) + ciphertext]
pub fn encrypt(
               data: &str, key: &[u8; 32]) 
               -> Result<String, Box<dyn Error>> {

    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, data.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    //Concatenate nonce and ciphertext
    let mut result = Vec::new();
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    //Perform base64 encoding
    let encoded_result = STANDARD.encode(&result);
    Ok(encoded_result)
}

// Decrypts data using AES-256-GCM
// Input is Base64 encoded encrypted text: [nonce (12 bytes) + ciphertext]
pub fn decrypt(
            encrypted_data: &str, key: &[u8; 32])
            ->Result<String, Box<dyn Error>> {

    //Decode the Base64 encoded text
    let decoded_data = match STANDARD.decode(encrypted_data) {
        Ok(data) => data,
        Err(error) => return Err(Box::new(error))
    };

    if decoded_data.len() < 12 {
        return Err("Encrypted data too short".into());
    }
    
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Nonce::from_slice(&decoded_data[..12]);
    let ciphertext = &decoded_data[12..];
    
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))?;
    
    String::from_utf8(plaintext)
        .map_err(|e| format!("Invalid UTF-8: {}", e).into())
}

//Function to display cleartext password on the screen:
//    Show the password for 'duration' secs
//    Clear it from the screen
//    Uses "crossterm" crate

pub fn print_password_cleartext(passwd: &str, duration: Duration)
       -> Result<Duration, Box<dyn Error>> {

    //Switch to raw mode
    terminal::enable_raw_mode()?;
    execute!(stdout(), cursor::SavePosition)?;
    print!("{:?}", passwd);
    stdout().flush()?;

    //Poll for key strokes OR until duration has elapsed
    let now = Instant::now();
    loop {
        //Break if duration has elapsed
        if now.elapsed() >= duration {
            break;
        }

        //Break if a key is pressed
        if crossterm::event::poll(Duration::from_millis(100))? {
            let _ = crossterm::event::read()?;
            break;
        }
    }
    execute!(stdout(), cursor::RestorePosition)?;
    execute!(stdout(), terminal::Clear(terminal::ClearType::FromCursorDown))?;
    stdout().flush()?;

    //Restore terminal mode
    terminal::disable_raw_mode()?;

    Ok(now.elapsed())
}
       

// Parses a raw credentials file and builds a CredentialStore HashMap.
//
// # Arguments
//
// * `raw_file_name` - 
// Path to the file containing raw credentials, with each line formatted as:
// <site> <user> <username> <password>.
//
// # Returns
//
// * `Ok(CredentialStore)` - A populated CredentialStore HashMap on success.
// * `Err(Box<dyn Error>)` - An error if the file cannot be read or parsed.
//
pub fn populate_db(raw_file_name: String, master_key: &[u8; 32]) -> 
                   Result<CredentialStore, Box<dyn Error>> { 
    // Read the file content 
    // TODO: Modify to use BufReader, 
    // in order to avoid reading the entire content
    let file_content = match fs::read_to_string(raw_file_name) {
        Ok(contents) => contents,
        Err(error)   => return Err(Box::new(error)),
    };

    let mut db: HashMap<String, SiteUser> = HashMap::new();

    // 
    // Read file_content, one line at a time: 
    // <site> <user> <username> <password>
    for line in file_content.lines() {
        let mut tokens = line.split_whitespace();
        let site = tokens.next().unwrap().to_string();

        // Check if the site is already present in the HashMap
        if let Some(site_user_map) = db.get_mut(&site) { 
            site_user_map.insert(tokens.next().unwrap().to_string(), 
                Credential {
                    username: tokens.next().unwrap().to_string(), 
                    password: encrypt(tokens.next().unwrap(), master_key)
                             .map_err(|e| format!("Encryption failed {}", e))?
                }
            );
        } 
        else {
            let mut site_user_map = HashMap::new();
            site_user_map.insert(tokens.next().unwrap().to_string(), 
                Credential {
                    username: tokens.next().unwrap().to_string(), 
                    password: encrypt(tokens.next().unwrap(), master_key)
                             .map_err(|e| format!("Encryption failed {}", e))?
                }
            );
            db.insert(site, site_user_map);
        };
    };

    Ok(db)
}
