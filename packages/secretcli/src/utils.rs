use colored::*;
use crate::{
    cli_types::{NetContract, TxQuery},
    secretcli::{test_contract_handle, init_cache},
};
use serde::Serialize;
use std::fmt::Display;

pub fn print_header(header: &str) {
    println!("{}", header.on_blue());
}

pub fn print_warning(warn: &str) {
    println!("{}", warn.on_yellow());
}

pub fn print_contract(contract: &NetContract) {
    println!(
        "\tLabel: {}\n\tID: {}\n\tAddress: {}\n\tHash: {}",
        contract.label, contract.id, contract.address, contract.code_hash
    );
}

pub fn print_struct<Printable: Serialize>(item: Printable) {
    println!("{}", serde_json::to_string_pretty(&item).unwrap());
}

pub fn print_vec<Type: Display>(prefix: &str, vec: Vec<Type>) {
    for e in vec.iter().take(1) {
        print!("{}{}", prefix, e);
    }
    for e in vec.iter().skip(1) {
        print!(", {}", e);
    }
    println!();
}

pub fn test_contract_init_and_debug<Message: serde::Serialize>(
    msg: &Message,
    file: &str,
    sender: &str,
    store_gas: Option<&str>,
    init_gas: Option<&str>,
    backend: Option<&str>,
    name: Option<&str>,
) {
    let result = init_cache(
        &msg,
        file,
        &generate_label(8),
        sender,
        store_gas,
        init_gas,
        backend,
        name,
    );

    match result {
        Ok(contract) => {
            println!("Contract address: {}", contract.address);
            println!("Contract code hash: {}", contract.code_hash);
        }
        Err(err) => {
            println!("{}", err);
        }
    }
}

pub fn test_contract_handle_and_debug<Message: serde::Serialize>(
    msg: &Message,
    contract: &NetContract,
    sender: &str,
    gas: Option<&str>,
    backend: Option<&str>,
    amount: Option<&str>,
) {
    let result = test_contract_handle(&msg, contract, sender, gas, backend, amount);

    match result {
        Ok((compute, query)) => {
            println!("{} {}", query.gas_used, query.gas_wanted);
            println!("ComputeResponse {}", compute.input);
        }
        Err(err) => {
            println!("{}", err);
        }
    }
}

pub fn assert_handle_failure(tx_query: TxQuery) -> bool {
    tx_query.raw_log.contains("failed to execute message")
}


pub const LABEL_ALPHABET: [char; 62] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
    'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

pub fn generate_label(size: usize) -> String {
    nanoid::nanoid!(size, &LABEL_ALPHABET)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gen_label() {
        let length: usize = 20;
        assert_eq!(length, generate_label(length).capacity())
    }
}
