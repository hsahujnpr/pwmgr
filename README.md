# PWMGR - Password Manager

## Overview
PWMGR is a secure and efficient password manager for personal use, written in Rust. It provides functionality for storing, encrypting, and managing credentials for multiple users and websites. The application uses AES-256-GCM encryption and SHA-256 hashing to ensure the security of stored passwords.

## Intended Use
PWMGR is designed to help users securely manage their passwords stored in a private file. The workflow involves two main steps:

1. **Initial Conversion**:
   - Use the `-r` option to read a raw credentials file and convert it into a structured JSON format.
   - The passwords in the JSON file are encrypted using a master password key.
   - Example:
     ```bash
     pwmgr -r raw_credentials.dat -d encrypted_credentials.json
     ```
     This command reads `raw_credentials.dat` and writes the encrypted data to `encrypted_credentials.json`.

2. **Password Management**:
   - Use PWMGR to manage the encrypted credentials with commands like `add`, `get`, `delete`, and `list`.
   - Example:
     ```bash
     pwmgr -d encrypted_credentials.json add gmail self me@gmail 
     pwmgr -d encrypted_credentials.json get gmail self
     pwmgr -d encrypted_credentials.json update gmail self me@gmail
     pwmgr -d encrypted_credentials.json delete gmail self
     pwmgr -d encrypted_credentials.json list
     pwmgr -d encrypted_credentials.json list-sites (future)
     pwmgr -d encrypted_credentials.json list-users site (future)
     ```

## Features
- **Secure Storage**: Credentials are encrypted using AES-256-GCM.
- **Master Password**: Derive a secure master key using SHA-256 hashing.
- **Credential Management**: Store and retrieve credentials for multiple users and websites.
- **Command-Line Interface**: Interact with the password manager via CLI.

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

## Usage
### Derive Master Key
Use the `derive_master_key` function to generate a secure master key from your master password.

### Encrypt and Decrypt Data
- Use the `encrypt` function to encrypt sensitive data.
- Use the `decrypt` function to decrypt previously encrypted data.

### Populate Credential Database
Use the `populate_db` function to parse a raw credentials file and build a secure credential store.

### Run Tests
Run unit tests to verify functionality:
```bash
cargo test
```

## Data Structure

### CredentialStore
The `CredentialStore` is implemented as a nested `HashMap`:
- **Outer HashMap**: Keys are website names (e.g., "gmail").
- **Inner HashMap**: Keys are users associated with the website, and values are `Credential` objects containing the username and encrypted password.

### Reasoning
The choice of a nested `HashMap` provides:
- **Efficient Lookups**: Both websites and users can be accessed in constant time (`O(1)`).
- **Hierarchical Organization**: Credentials are stored in a way that supports multiple users per website, ensuring that each user’s credentials are isolated and easily retrievable.
- **Scalability**: Handles large numbers of credentials without significant performance degradation.

This structure ensures that credentials are organized hierarchically, making it easy to manage and retrieve data for specific websites and their associated users. Additionally, the use of `HashMap` aligns with Rust's standard library, ensuring reliability and ease of use.

## File Structure
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

## Example Usage

#### Raw Credentials File Format
The raw credentials file should be formatted as follows:

```plaintext
<site> <user> <username> <password>
```

#### Sample Data
Here is an example of a raw credentials file:

```plaintext
test-site test-user test-username test-password
hdfcbank self myusername mypassword
hdfcbank mom mom-username mom-password
gmail self me@gmail mypassword@gmail
gmail mom mom@gmail mompassword@gmail
```

#### Populating CredentialStore
Using the `populate_db` function, the above data will be transformed into the following structure:

```rust
{
    "test-site": {
        "test-user": Credential {
            username: "test-username",
            password: "<encrypted-password>"
        }
    },
    "hdfcbank": {
        "self": Credential {
            username: "myusername",
            password: "<encrypted-password>"
        },
        "mom": Credential {
            username: "mom-username",
            password: "<encrypted-password>"
        }
    },
    "gmail": {
        "self": Credential {
            username: "me@gmail",
            password: "<encrypted-password>"
        },
        "mom": Credential {
            username: "mom@gmail",
            password: "<encrypted-password>"
        }
    }
}
```

Note: The `<encrypted-password>` placeholder represents the password encrypted using the master key derived from the `derive_master_key` function.

### Error Handling
Each command provides meaningful error messages if the operation fails. For example:
- Attempting to retrieve credentials for a non-existent site or user will return:
  ```plaintext
  No Credentials exist for Site: wrong-site User: wrong-user
  ```
- Attempting to add credentials for an user on existing site will return:
  ```plaintext
  Credentials exist for Site: site User: user - Use 'Update' instead
  ```

### Future Work
1. Prompt to enter new password twice, to avoid mistyping
2. Store supplemental data in Credential structure, e.g, some websites require a "profile password" for certain sections of the site
3. Mechanism to change master password (in certain intervals). When that happens entire credential database need to be re-encrypted with new key
4. Additional CLI commands indicated above
