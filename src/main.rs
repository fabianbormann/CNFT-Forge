use blockfrost::{load, Address, AddressUtxo, Block, BlockFrostApi, BlockFrostSettings};
use regex::Regex;
use rocket::fs::NamedFile;
use rocket::http::Status;
use rocket::serde::json::{json, Value};
use rocket::serde::{json::Json, Deserialize, Serialize};
use std::fs::{create_dir, read_to_string, remove_dir_all, remove_file, write, File};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
extern crate hex;

#[get("/<path..>")]
async fn index(path: PathBuf) -> Option<NamedFile> {
    let mut path = Path::new(&format!("{}/frontend/build", env!("CARGO_MANIFEST_DIR"))).join(path);

    if path.is_dir() {
        path.push("index.html");
    }

    NamedFile::open(path).await.ok()
}

#[macro_use]
extern crate rocket;
#[get("/address/<address>")]
async fn address_info(address: String) -> Value {
    let address_info = get_address_info(address).await;

    if let Err(_err) = address_info {
        println!("{:?}", _err);
        json!("Error")
    } else {
        json!(address_info.unwrap())
    }
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Metadata {
    name: String,
    description: String,
    amount: u16,
    payment_address: String,
    ipfs_cid: String,
    verification_key: String,
    signing_key: String,
}

#[post("/nft", format = "json", data = "<metadata>")]
async fn build_nft(metadata: Json<Metadata>) -> (Status, Value) {
    let tokenname = hex::encode(&metadata.name);
    let utxos = query_utxos(&metadata.payment_address).await;
    let latest_blocks = query_tip().await;
    let slot = latest_blocks.unwrap().slot.unwrap() + 10000;

    let address_utxos = utxos.unwrap();
    let address_utxo: &AddressUtxo = address_utxos.first().unwrap();
    let tx_hash = &address_utxo.tx_hash;
    let tx_ix = address_utxo.output_index.to_string();

    let status = create_policy_id(slot).await;

    if status.success() {
        let path = &format!("{}/policy/policyID", env!("CARGO_MANIFEST_DIR"));
        let policy_id_from_file =
            read_to_string(path).expect("Unable to read policy id from file.");
        let policy_id = policy_id_from_file.trim();

        let metadata_json = &format!(
            r#"{{
            "721": {{
                "{}": {{
                  "NFT1": {{
                    "description": "{}",
                    "name": "{}",
                    "id": 1,
                    "image": "ipfs://{}"
                  }}
                }}
            }}
        }}"#,
            policy_id, metadata.description, metadata.name, metadata.ipfs_cid
        );

        let metadata_json_path = &format!("{}/metadata.json", env!("CARGO_MANIFEST_DIR"));

        if Path::new(metadata_json_path).exists() {
            let action_result = remove_file(metadata_json_path);
            if action_result.is_err() {
                println!("Unable to remove old metadata.json file");
            }
        }

        write(metadata_json_path, metadata_json).unwrap();

        let output = 1400000;
        let policy_script_path = &format!("{}/policy/policy.script", env!("CARGO_MANIFEST_DIR"));
        let build_transaction_output = build_transaction(
            tx_hash,
            tx_ix.as_str(),
            metadata.payment_address.as_str(),
            output,
            metadata.amount,
            policy_id,
            tokenname.as_str(),
            policy_script_path.as_str(),
            slot,
        )
        .await;

        let transaction_requirements = String::from_utf8(build_transaction_output.stdout).unwrap();
        let build_transaction_stderr = String::from_utf8(build_transaction_output.stderr).unwrap();

        if transaction_requirements.starts_with("Minimum required UTxO") {
            println!("{}", transaction_requirements);

            let regex = Regex::new(r"/d+").unwrap();
            let lovelace_match = regex.find(transaction_requirements.as_str()).unwrap();
            println!("{}", lovelace_match.as_str());
            // TODO: update output
        }

        if transaction_requirements.starts_with("Estimated transaction fee") {
            println!("{}", transaction_requirements);
            let (sign_and_submit_status, sign_and_submit_message) =
                sign_and_submit_transaction().await;

            if sign_and_submit_status.success() {
                (
                    Status::Ok,
                    json!({
                        "tokenname": metadata.name,
                        "policyId": policy_id,
                        "submitOutput": sign_and_submit_message
                    }),
                )
            } else {
                (
                    Status::InternalServerError,
                    json!({"error": "Failed to sign and submit the transaction"}),
                )
            }
        } else {
            println!("{}", transaction_requirements);
            (
                Status::InternalServerError,
                json!({
                    "error": "Could not estimate the required lovelace output",
                    "message": build_transaction_stderr
                }),
            )
        }
    } else {
        (
            Status::InternalServerError,
            json!({"error": "Could not create a policy id"}),
        )
    }
}

async fn create_policy_id(slot: i128) -> ExitStatus {
    let policy_dir = &format!("{}/policy", env!("CARGO_MANIFEST_DIR"));

    if Path::new(policy_dir).exists() {
        let action_result = remove_dir_all(policy_dir);
        if action_result.is_err() {
            println!("Unable to remove old policy dir");
        }
    }

    create_dir(policy_dir).expect("Failed to create policy dir");

    let key_gen_status = Command::new("cardano-cli")
        .args([
            "address",
            "key-gen",
            "--verification-key-file",
            &format!("{}/policy/policy.vkey", env!("CARGO_MANIFEST_DIR")),
            "--signing-key-file",
            &format!("{}/policy/policy.skey", env!("CARGO_MANIFEST_DIR")),
        ])
        .status()
        .expect("Failed to create a policy key pairs");

    if !key_gen_status.success() {
        return key_gen_status;
    }

    let key_hash_generation = Command::new("cardano-cli")
        .args([
            "address",
            "key-hash",
            "--payment-verification-key-file",
            &format!("{}/policy/policy.vkey", env!("CARGO_MANIFEST_DIR")),
        ])
        .output()
        .unwrap();

    let key_hash = String::from_utf8(key_hash_generation.stdout).unwrap();

    let policy_script = &format!(
        r#"{{
        "type": "all",
        "scripts":
        [
          {{
            "type": "before",
            "slot": {}
          }},
          {{
            "type": "sig",
            "keyHash": "{}"
          }}
        ]
      }}"#,
        slot,
        key_hash.trim()
    );

    let policy_script_path = &format!("{}/policy/policy.script", env!("CARGO_MANIFEST_DIR"));
    write(policy_script_path, policy_script).unwrap();

    let path = &format!("{}/policy/policyID", env!("CARGO_MANIFEST_DIR"));
    let policy_id_file = File::create(path).expect(&format!("Failed to open file {}", path));

    Command::new("cardano-cli")
        .args([
            "transaction",
            "policyid",
            "--script-file",
            policy_script_path,
        ])
        .stdout(policy_id_file)
        .status()
        .expect("Failed to create a policy id")
}

async fn build_transaction(
    tx_hash: &str,
    tx_id: &str,
    payment_address: &str,
    output: i128,
    token_amount: u16,
    policy_id: &str,
    token_name: &str,
    policy_script_path: &str,
    slot: i128,
) -> Output {
    let out_file_path = &format!("{}/matx.raw", env!("CARGO_MANIFEST_DIR"));

    Command::new("cardano-cli")
        .args([
            "transaction",
            "build",
            "--testnet-magic",
            "1097911063",
            "--babbage-era",
            "--tx-in",
            format!("{}#{}", tx_hash, tx_id).as_str(),
            "--tx-out",
            format!(
                "{}+{}+{} {}.{}",
                payment_address, output, token_amount, policy_id, token_name
            )
            .as_str(),
            "--change-address",
            payment_address,
            format!("--mint={} {}.{}", token_amount, policy_id, token_name).as_str(),
            "--minting-script-file",
            policy_script_path,
            "--metadata-json-file",
            "metadata.json",
            "--invalid-hereafter",
            format!("{}", slot).as_str(),
            "--witness-override",
            "2",
            "--out-file",
            out_file_path,
        ])
        .stdout(Stdio::piped())
        .output()
        .expect("failed to execute transaction build process")
}

async fn sign_and_submit_transaction() -> (ExitStatus, String) {
    let payment_skey_file_path = &format!("{}/payment.skey", env!("CARGO_MANIFEST_DIR"));
    let policy_skey_file_path = &format!("{}/policy/policy.skey", env!("CARGO_MANIFEST_DIR"));
    let matx_file_path = &format!("{}/matx.raw", env!("CARGO_MANIFEST_DIR"));
    let matx_signed_file_path = &format!("{}/matx.signed", env!("CARGO_MANIFEST_DIR"));

    let signing_result = Command::new("cardano-cli")
        .args([
            "transaction",
            "sign",
            "--signing-key-file",
            payment_skey_file_path,
            "--signing-key-file",
            policy_skey_file_path,
            "--testnet-magic",
            "1097911063",
            "--tx-body-file",
            matx_file_path,
            "--out-file",
            matx_signed_file_path,
        ])
        .stdout(Stdio::piped())
        .output()
        .expect("failed to execute transaction signing process");

    if signing_result.status.success() {
        let submit_result = Command::new("cardano-cli")
            .args([
                "transaction",
                "submit",
                "--tx-file",
                matx_signed_file_path,
                "--testnet-magic",
                "1097911063",
            ])
            .stdout(Stdio::piped())
            .output()
            .expect("failed to execute transaction signing process");

        (
            submit_result.status,
            String::from_utf8(submit_result.stdout).unwrap(),
        )
    } else {
        return (
            signing_result.status,
            String::from_utf8(signing_result.stdout).unwrap(),
        );
    }
}

async fn query_utxos(address: &String) -> blockfrost::Result<Vec<AddressUtxo>> {
    let api = build_api()?;
    let address_utxo = api.addresses_utxos(address).await;

    Ok(address_utxo.unwrap())
}

async fn query_tip() -> blockfrost::Result<Block> {
    let api = build_api()?;
    let latest_blocks = api.blocks_latest().await;
    Ok(latest_blocks.unwrap())
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, address_info, build_nft])
}

fn build_api() -> blockfrost::Result<BlockFrostApi> {
    let configurations = load::configurations_from_env()?;

    let project_id = configurations["project_id"].as_str().unwrap();
    let cardano_network: String = String::from(configurations["cardano_network"].as_str().unwrap());

    let settings = BlockFrostSettings {
        network_address: cardano_network,
        query_parameters: Default::default(),
        retry_settings: Default::default(),
    };

    let api = BlockFrostApi::new(project_id, settings);
    Ok(api)
}

async fn get_address_info(address: String) -> blockfrost::Result<Address> {
    let api = build_api()?;
    let address_info = api.addresses(&address).await;

    Ok(address_info.unwrap())
}
