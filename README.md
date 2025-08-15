# PWMGR - Password Manager in Rust

## Overview
PWMGR is a simple password manager tool for personal use. It provides functionality for storing, and managing credentials for all websites for the user. You can store credentials for multiple users for the same website (e.g., my own credential and my mom's credential for, let's say, gmail.com). 

## Features
Here are the key features of PWMGR:
   - It is reliable and memory-safe, since it is implemented in Rust. 
   - It uses AES-256-GCM encryption with random nonces for encrypting passwords.
   - It uses a master password (default: "pwmgr123"), 
     which should be changed before adding more credentials to manage. 
     Credential passwords are encrypted using key derived from the password.

## Usage
There are three data files:
---------------------------
   - **data/cred_db.json**: 
     This is the credential database, stores the credentials in JSON format. 
     The password fields are encrypted.

   - **data/pwmgr_master_hash**: 
     Stores the base64-encoded hash of the master password ("pwmgr123"). 
     Before adding new credentials, change the master password:
     ```bash
     pwmgr -d data/cred_db.json -m data/pwmgr_master_hash set-master-password
     ```
     This will change master password, re-encrypt the credentials in 
     cred_db.json with encryption key derived from the new master password.

   - **raw_credentials.dat**: 
     Example 'raw' credentials, used to build the initial credential database.
     If you have raw credentials stored in some text file, you can add them in 
     this file, in the same format, and use pwmgr with "-r" option to import 
     them all into the credential database.
     ```bash
     pwmgr -r data/raw_credentials.dat -d data/cred_db.json -m data/pwmgr_master_hash list
     ```
     Note that, this will overwrite the cred_db.json (not append to it). 
     So, any raw credentials needs to be converted using the "-r" option first, 
     before adding new ones into the database.

Managing the Credential database:
---------------------------------
   - Use command line options to manage the encrypted credentials as follows:
   - Example:
     ```bash
     pwmgr -d data/cred_db.json -m data/pwmgr_master_hash list
     pwmgr -d data/cred_db.json -m data/pwmgr_master_hash add gmail self me@gmail 
     pwmgr -d data/cred_db.json -m data/pwmgr_master_hash get gmail self
     pwmgr -d data/cred_db.json -m data/pwmgr_master_hash update gmail self me@gmail
     pwmgr -d data/cred_db.json -m data/pwmgr_master_hash delete gmail self
     pwmgr -d data/cred_db.json -m data/pwmgr_master_hash list-sites (future)
     pwmgr -d data/cred_db.json -m data/pwmgr_master_hash list-users site (future)
     ```

## Installation
1. Clone the repository:
   ```bash
   git clone <repository-url>
   ```
2. Navigate to the project directory:
   ```bash
   cd pwmgr
   ```
3. Build the project using Cargo:
   ```bash
   cargo build --release
   ```

## Implementation
### CredentialStore
The `CredentialStore` is implemented as a nested `HashMap`:
- **Outer HashMap**: Keys are website names (e.g., "gmail").
- **Inner HashMap**: Keys are users associated with the website, and values are `Credential` objects containing the username and encrypted password.

### Rationale
The choice of a nested `HashMap` provides:
- **Efficient Lookups**: Both websites and users can be accessed in constant time (`O(1)`).
- **Hierarchical Organization**: Credentials are stored in a way that supports multiple users per website, ensuring that each user’s credentials are isolated and easily retrievable.
- **Scalability**: Handles large numbers of credentials without significant performance degradation.

This structure ensures that credentials are organized hierarchically, making it easy to manage and retrieve data for specific websites and their associated users. Additionally, the use of `HashMap` aligns with Rust's standard library, ensuring reliability and ease of use.

## File Structure
- **src/structs.rs**: Contains the Credential structure.
- **src/lib.rs**: Contains core cryptographic functions and credential management logic.
- **src/main.rs**: CLI entry point for the password manager.
- **data/**: Contains raw and encrypted credential files.

## Dependencies
- `aes-gcm`: For AES-256-GCM encryption.
- `sha2`: For SHA-256 hashing.
- `serde`: For serialization and deserialization.
- `rand`: For generating random nonces.

## Contributing
Contributions are welcome! Please submit a pull request or open an issue for any bugs or feature requests.

## License
This project is licensed under the MIT License. See the LICENSE file for details.

## Disclaimer
PWMGR is provided as-is without any warranty. Use it at your own risk.

### Future Work
1. Store supplemental data in Credential structure, e.g, some websites require a "profile password" for certain sections of the site
2. Additional CLI commands indicated above
