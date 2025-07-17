use std::process;
use std::collections::HashMap;
use clap::{Parser, Subcommand};
use std::fs;
use pwmgr::structs::Credential;
use rpassword;
//use log::{debug, info, warn};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    //Specify  a file name containing raw credentials
    #[arg(short, long, value_name="RAW_FILE_NAME")]
    raw_cred_file_name: Option<String>,

    //Specify the name of DB file name
    #[arg(short, long, value_name="DB_FILE_NAME")]
    db_file_name: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add    {site: String, user: String, username: String},
    Get    {site: String, user: String},
    Update {site: String, user: String, username: String},
    Delete {site: String, user: String},
    List   {},
}

fn main() {
    println!("Welcome to Credential manager!");
    println!("==============================");

    //Get master password
    let master_password = 
        rpassword::prompt_password("Enter Master Password:").unwrap();

    let master_key = match pwmgr::derive_master_key(&master_password){
        Ok(key) => {
            key
        },
        Err(error) => {
                println!("{}",error.to_string());
                process::exit(1);
        },
    };

    let args = Cli::parse();

    let mut cred_db = if let Some(raw_file_name) = args.raw_cred_file_name {
        println!("Raw Credentials file name: {:?}", raw_file_name);
        //Load it in the 'cred_db' hashmap
        let db = match pwmgr::populate_db(raw_file_name, &master_key){
            Err(_error) => {
                println!("Could not construct Hashmap from raw credentials!");
                process::exit(1);
            },
            Ok(db) => db
        };
        db
    }
    else {
        //Load the cred_db hashmap from args.db_file_name
        let db = match fs::read_to_string(&args.db_file_name) {
            Ok(db_file_content) => {
                let db = serde_json::from_str(&db_file_content).unwrap();
                db
            },
            Err(error) => {
                println!("File Error: {error}, creating new Hashmap");
                let db:pwmgr::CredentialStore = HashMap::new();
                db
            },
        };
        db
    };

    //Implement actions on the credential DB here
    match args.command {
        Commands::List {} => {
            for (site, site_users) in cred_db.iter() {
                println!("Site: {:?}", site);
                for (user, cred) in site_users.iter() {
                    println!("    User: {:?} Credentials: {:?}", user, cred);
                }
            }
        }
        Commands::Add {site, user, username} => {
            let new_pass = 
                rpassword::prompt_password("Enter Password:").unwrap();
            let new_encrypted_pass = pwmgr::encrypt(&new_pass, &master_key);
            if let Some(site_user) = cred_db.get_mut(&site) {
                if site_user.contains_key(&user) {
                    println!(
                    "Credentials exist for Site: {:?} User: {:?} - 
                    Use 'Update' instead", 
                    site, user);
                }
                else {
                    println!(
                    "Adding new user for Site: {:?} User: {:?}", 
                    site, user);

                    site_user.insert(
                        user, 
                        Credential{username:username, 
                        password:new_encrypted_pass}
                    );
                }
            }
            else {
                println!(
                "Adding new site: {:?} new user: {:?}", 
                site, user);

                let mut site_users = HashMap::new();
                site_users.insert(
                    user, 
                    Credential{username:username, 
                        password:new_encrypted_pass}
                );
                cred_db.insert(site, site_users);
            }
        }

        Commands::Delete {site, user} => {
            if let Some(site_user) = cred_db.get_mut(&site) {
                if !site_user.contains_key(&user) {
                    println!(
                    "No Credentials exist for Site: {:?} User: {:?} - 
                    Nothing to delete!", site, user);
                }
                else {
                    println!(
                    "Removing Credentials for Site: {:?} User: {:?}", 
                    site, user);
                    site_user.remove(&user);
                }
            }
            else {
                println!(
                "No Credentials exist for this Site - Nothing to delete!" );
            }
        }

        Commands::Get {site, user} => {
            if let Some(site_user) = cred_db.get(&site) {
                if let Some(cred) = site_user.get(&user) {
                    match pwmgr::decrypt(&cred.password, &master_key){
                        Ok(plaintext) => {
                            println!(
                            "Credentials for Site: {:?} User: {:?}: 
                            username: {:?}, Password: {:?}", 
                            site, user, cred.username, plaintext);

                            plaintext
                        },
                        Err(err_msg) => {
                            println!("{}", err_msg);
                            process::exit(1);
                        }
                    };
                }
                else {
                    println!(
                    "No Credentials exist for Site: {:?} User: {:?}!", 
                    site, user);
                }
            }
            else {
                println!("No Credentials exist for this Site!" );
            }
        }

        Commands::Update {site, user, username} => {
            if let Some(site_user) = cred_db.get_mut(&site) {
                if let Some(_cred) = site_user.get_mut(&user) {
                    println!(
                    "Updating Credentials for Site: {:?} User: {:?}, ", 
                    site, user);

                    let new_pass = rpassword::prompt_password(
                    "Enter Password:").unwrap();

                    let new_encrypted_pass = 
                        pwmgr::encrypt(&new_pass, &master_key);

                    site_user.insert(
                        user, 
                        Credential{
                            username:username, password:new_encrypted_pass}
                    );
                }
                else {
                    println!(
                    "No Credentials exist for Site: {:?} User: {:?} -           
                    Nothing to update!", site, user);
                }
            }
            else {
                println!("No Credentials exist for this Site - 
                Nothing to update!");
            }
        }
    }

    //Save DB to file in JSON format
    let db_file_content = 
        serde_json::to_string_pretty(&cred_db).
        expect("Failed to serialize DB");

    let _ = fs::write(args.db_file_name, db_file_content);
}
