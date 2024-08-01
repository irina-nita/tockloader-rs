mod board_attributes;
mod board_settings;
mod errors;
mod kernel_attributes;

use std::collections::HashMap;

use board_attributes::{get_all_attributes, get_bootloader_version};
use board_settings::BoardSettings;
use kernel_attributes::kernel_attributes;

use clap::ArgMatches;
use errors::TockloaderError;
use inquire::Select;
use probe_rs::probe::list::Lister;
use probe_rs::{MemoryInterface, Permissions};
use tbf_parser::parse::*;
use tbf_parser::types::*;

pub async fn list_probe(
    sub_matches: &ArgMatches,
) -> Result<Vec<HashMap<String, String>>, TockloaderError> {
    let lister = Lister::new();
    let probes = lister.list_all();
    println!("Probes: {:?}\n", probes);

    let ans = Select::new("Which probe do you want to use?", probes).prompt();

    let mut apps_details: Vec<HashMap<String, String>> = vec![];

    match ans {
        Ok(choice) => {
            let probe = choice.open().unwrap();

            let chip = sub_matches.get_one::<String>("chip").unwrap();
            let board = sub_matches.get_one::<String>("board").unwrap();

            let board_settings = BoardSettings::new(board.clone(), chip.clone());

            let mut session = probe
                .attach(board_settings.chip, Permissions::default())
                .unwrap();

            let core_index = sub_matches.get_one::<usize>("core").unwrap();

            let mut core = session.core(*core_index).unwrap();

            let mut address = board_settings.start_address;

            // Jump through the linked list of apps
            let mut apps_counter = 0;
            loop {
                let mut temp_details: HashMap<String, String> = HashMap::new();

                // Read a block of 200 8-bit words
                let mut buff = vec![0u8; 200];
                match core.read(address, &mut buff) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error reading memory: {:?}", e);
                        break;
                    }
                }

                let (ver, header_size, total_size) =
                    match parse_tbf_header_lengths(&buff[0..8].try_into().unwrap()) {
                        Ok((ver, header_size, total_size)) if header_size != 0 => {
                            temp_details.insert("version".to_owned(), ver.to_string());
                            temp_details.insert("header_size".to_owned(), header_size.to_string());
                            temp_details.insert("total_size".to_owned(), total_size.to_string());
                            (ver, header_size, total_size)
                        }
                        _ => break, // No more apps
                    };

                let header = parse_tbf_header(&buff[0..header_size as usize], ver);
                match header {
                    Ok(header) => {
                        temp_details.insert("enabled".to_owned(), header.enabled().to_string());
                        temp_details.insert(
                            "minumum_ram_size".to_owned(),
                            header.get_minimum_app_ram_size().to_string(),
                        );
                        temp_details.insert(
                            "name".to_owned(),
                            header
                                .get_package_name()
                                .expect("Package name not found.")
                                .to_string(),
                        );
                        temp_details.insert(
                            "kernel_version".to_owned(),
                            format!(
                                "{}.{}",
                                header
                                    .get_kernel_version()
                                    .expect("Kernel version not found.")
                                    .0,
                                header
                                    .get_kernel_version()
                                    .expect("Kernel version not found.")
                                    .1
                            ),
                        );
                    }
                    // TODO(MicuAna): refactor when reworking errors
                    Err(TbfParseError::ChecksumMismatch(
                        provided_checksum,
                        calculated_checksum,
                    )) => {
                        println!(
                            "Checksum mismatch: provided = {}, calculated = {}",
                            provided_checksum, calculated_checksum
                        );
                        break;
                    }
                    Err(e) => {
                        println!("Failed to parse TBF header: {:?}", e);
                        break;
                    }
                }
                apps_details.insert(apps_counter, temp_details);
                apps_counter += 1;
                address += total_size as u64;
            }
        }
        Err(err) => println!("{}", err),
    }

    Ok(apps_details)
}

pub async fn info_probe(sub_matches: &ArgMatches) -> Result<(), TockloaderError> {
    let lister = Lister::new();
    let probes = lister.list_all();

    let ans = Select::new("Which probe do you want to use?", probes).prompt();
    match ans {
        Ok(choice) => {
            let probe = choice.open().unwrap();

            let chip = sub_matches.get_one::<String>("chip").unwrap();
            let board = sub_matches.get_one::<String>("board").unwrap();

            let board_settings = BoardSettings::new(board.clone(), chip.clone());

            let mut session = probe
                .attach(board_settings.chip, Permissions::default())
                .unwrap();

            let core_index = sub_matches.get_one::<usize>("core").unwrap();

            let mut core = session.core(*core_index).unwrap();

            let mut attributes = get_all_attributes(&mut core);

            let bootloader_version = get_bootloader_version(&mut core);

            kernel_attributes(&mut core, &mut attributes);

            attributes.insert("bootloader_version".to_owned(), bootloader_version);

            println!("{:?}", attributes);

            // println!("Bootloader Version: {}", bootloader_version);

            Ok(())
        }
        Err(err) => {
            println!("While picking probe:{}", err);
            //TODD(NegrilaRares) JUST TEMPLATE ERROR MUST BE CHANGED LATER
            Err(TockloaderError::NoPortAvailable)
        }
    }
}
