use std::fs;
use std::path::PathBuf;
use assert_cmd::Command;
use predicates::prelude::predicate;


#[test]
fn verify_online_commands() -> Result<(), Box<dyn std::error::Error>> {
    // setup
    let _ = remove_wallet("default")?;
    let _ = remove_wallet("test55")?;
    let _ = remove_wallet("test66")?;
    // Test basic commands
    verify_help(vec!("help","-h","--help"))?;
    verify_version(vec!("--version","-V"))?;
    // Test online wallet
    verify_create_new_wallet("default")?;
    verify_get_balance("default")?;
    verify_list_transactions("default")?;
    verify_new_address("default")?;
    verify_send("default")?;
    // Test named online wallet
    verify_create_new_wallet("test55")?;
    verify_get_balance("test55")?;
    verify_list_transactions("test55")?;
    verify_new_address("test55")?;
    verify_send("test55")?;
    // Import seed wallet by seeds
    verify_import_new_wallet("test66")?;

    Ok(())
}

fn verify_help(flags : Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    for flag in flags {
        let mut cmd = Command::cargo_bin("btc-tool") ?;

        cmd.env("BTC_TOOL_PWD",
        "asdfasdf")
        .env("BTC_TOOL_HOME",
        "target/tmp")
        .arg(flag);


        cmd.assert()
        .success()
        .stdout(predicate::str::contains("USAGE:"))
        .stdout(predicate::str::contains("OPTIONS:"))
        .stdout(predicate::str::contains("SUBCOMMANDS:"));

    }
    Ok(())
}

fn verify_version(flags : Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    for flag in flags {
        let mut cmd = Command::cargo_bin("btc-tool")?;

        cmd.env("BTC_TOOL_PWD", "asdfasdf")
            .env("BTC_TOOL_HOME", "target/tmp")
            .arg(flag);


        cmd.assert()
            .success()
            .stdout(predicate::str::contains("btc-tool"));
    }

    Ok(())
}

fn verify_create_new_wallet(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("btc-tool")?;

    cmd.env("BTC_TOOL_PWD","asdfasdf")
        .env("BTC_TOOL_HOME","target/tmp")
        .arg("create");

    if name != "default" {
        cmd.arg("--name").arg(name);
    }

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!("You are about to generate a new wallet with name {}.",name)))
        .stdout(predicate::str::contains("New Seed generated:"))
        .stdout(predicate::str::contains(format!("Wallet created and stored in target/tmp/{}.wallet",name)));


    assert!(get_db_file(name).exists());
    assert!(get_wallet_file(name).exists());

    Ok(())
}

fn verify_import_new_wallet(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("btc-tool")?;

    cmd.env("BTC_TOOL_PWD","asdfasdf")
        .env("BTC_TOOL_HOME","target/tmp")
        .arg("import");

    if name != "default" {
        cmd.arg("--name").arg(name);
    }

    let mut stdin_buff = vec![];
    // First word is wrong and should prompt reentry
    stdin_buff.push("blir\n");
    // Enter all seed 12 words
    stdin_buff.push("general\nvessel\npayment\nvintage\npatch\nroyal\nsituate\nuntil\ngift\ndefy\nlock\ndisease\n");
    // Enter invalid confirmation then want to enter seed phrases again
    stdin_buff.push("invalid\nno\n");
    // Enter seed phrases again, but word 2 wrong this time
    stdin_buff.push("general\nvkssel\nvessel\npayment\nvintage\npatch\nroyal\nsituate\nuntil\ngift\ndefy\nlock\ndisease\n");
    // Confirm
    stdin_buff.push("yes\n");
    cmd.write_stdin(stdin_buff.join(""));

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(format!("You are about to recreate a new wallet with name {}.",name)))
        .stdout(predicate::str::contains("Enter your seed phrases (Use Ctrl-C to abort)"))
        .stdout(predicate::str::contains("Enter word 1:"))
        .stdout(predicate::str::contains("Invalid word 1 entered, try again"))
        .stdout(predicate::str::contains("Enter word 12:"))
        .stdout(predicate::str::contains("You have entered the following seed phrases:"))
        .stdout(predicate::str::contains("Is this correct? (yes,no):"))
        .stdout(predicate::str::contains("Invalid input, enter 'yes' or 'no'."))
        .stdout(predicate::str::contains("Invalid word 2 entered, try again"))
        .stdout(predicate::str::contains(format!("Wallet created and stored in target/tmp/{}.wallet",name)));


    assert!(get_db_file(name).exists());
    assert!(get_wallet_file(name).exists());

    Ok(())
}

fn verify_get_balance(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("btc-tool")?;

    cmd.env("BTC_TOOL_PWD","asdfasdf")
        .env("BTC_TOOL_HOME","target/tmp")
        .arg("get-balance");

    if name != "default" {
        cmd.arg("--name").arg(name);
    }

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Synchronizing Blockchain..."))
        .stdout(predicate::str::contains("Sync Complete."))
        .stdout(predicate::str::contains("Current balance:"));

    Ok(())
}

fn verify_list_transactions(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("btc-tool")?;

    cmd.env("BTC_TOOL_PWD","asdfasdf")
        .env("BTC_TOOL_HOME","target/tmp")
        .arg("list-transactions");

    if name != "default" {
        cmd.arg("--name").arg(name);
    }

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Synchronizing Blockchain..."))
        .stdout(predicate::str::contains("Sync Complete."))
        .stdout(predicate::str::contains("TransactionId"));

    Ok(())
}

fn verify_new_address(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("btc-tool")?;

    cmd.env("BTC_TOOL_PWD","asdfasdf")
        .env("BTC_TOOL_HOME","target/tmp")
        .arg("new-address");

    if name != "default" {
        cmd.arg("--name").arg(name);
    }

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("New address: "));

    Ok(())
}

fn verify_send(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    verify_send_invalid_arguments(&name)?;
    verify_send_with_no_fee_argument(&name)?;
    verify_send_with_fee_argument(&name)?;
    Ok(())
}

fn verify_send_invalid_arguments(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("btc-tool")?;

    cmd.env("BTC_TOOL_PWD","asdfasdf")
        .env("BTC_TOOL_HOME","target/tmp")
        .arg("send");

    if name != "default" {
        cmd.arg("--name").arg(name);
    }

    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("he following required arguments were not provided:"))
        .stderr(predicate::str::contains("--address <ADDRESS>"))
        .stderr(predicate::str::contains("--amount <AMOUNT>"));

    Ok(())
}

fn verify_send_with_no_fee_argument(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("btc-tool")?;

    cmd.env("BTC_TOOL_PWD","asdfasdf")
        .env("BTC_TOOL_HOME","target/tmp")
        .arg("send")
        .arg("--address").arg("tb1qre7567v42sa2gu24g42datxvpkl0pkxm6vte7n")
        .arg("--amount").arg("1000");

    if name != "default" {
        cmd.arg("--name").arg(name);
    }

    cmd.assert()
        .failure()
        .code(253)
        .stdout(predicate::str::contains("Synchronizing Blockchain..."))
        .stdout(predicate::str::contains("Sync Complete."))
        .stdout(predicate::str::contains("Error occurred executing command:InsufficientFunds"));

    Ok(())
}

fn verify_send_with_fee_argument(name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("btc-tool")?;

    cmd.env("BTC_TOOL_PWD","asdfasdf")
        .env("BTC_TOOL_HOME","target/tmp")
        .arg("send")
        .arg("--address").arg("tb1qre7567v42sa2gu24g42datxvpkl0pkxm6vte7n")
        .arg("--amount").arg("1000")
        .arg("--fee").arg("2");

    if name != "default" {
        cmd.arg("--name").arg(name);
    }

    cmd.assert()
        .failure()
        .code(253)
        .stdout(predicate::str::contains("Synchronizing Blockchain..."))
        .stdout(predicate::str::contains("Sync Complete."))
        .stdout(predicate::str::contains("Error occurred executing command:InsufficientFunds"))
        .stdout(predicate::str::contains("needed: 1082"));

    Ok(())
}

fn remove_wallet(name: &str) -> Result<(), Box<dyn std::error::Error>>{
    let db_file = get_db_file(name);
    let wallet_file = get_wallet_file(name);
    if db_file.exists(){
        fs::remove_dir_all(db_file)?;
    }
    if wallet_file.exists(){
        fs::remove_file(wallet_file)?;
    }
    Ok(())
}

fn get_db_file(name: &str) -> PathBuf {
    PathBuf::from(format!("target/tmp/{}.db",name))
}

fn get_wallet_file(name: &str) -> PathBuf {
    PathBuf::from(format!("target/tmp/{}.wallet",name))
}