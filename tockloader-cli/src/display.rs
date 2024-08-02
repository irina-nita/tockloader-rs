// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OXIDOS AUTOMOTIVE 2024.

use tockloader_lib::attributes::{
    app_attributes::AppAttributes, hardware_attributes::HardwareAttributes,
};

pub async fn print_list(app_details: &mut [AppAttributes], info: bool) {
    match info {
        false => {
            for (i, temp_data) in app_details.iter().enumerate() {
                println!("\n\x1b[0m\x1b[1;35m ┏━━━━━━━━━━━━━━━━┓");
                println!(
                    "\x1b[0m\x1b[1;31m ┃ \x1b[0m\x1b[1;32m App_{:<9?} \x1b[0m\x1b[1;31m┃",
                    i
                );
                println!("\x1b[0m\x1b[1;33m ┗━━━━━━━━━━━━━━━━┛");
                println!(
                    "\n \x1b[1;32m Name:                {}",
                    temp_data.name.clone().unwrap()
                );

                println!(
                    " \x1b[1;32m Enabled:             {}",
                    temp_data.enabled.unwrap()
                );

                println!(
                    " \x1b[1;32m Total_Size:          {}\n\n",
                    temp_data.total_size.unwrap()
                );
            }
        }

        true => {
            for (i, temp_data) in app_details.iter().enumerate() {
                println!("\n\x1b[0m\x1b[1;35m ┏━━━━━━━━━━━━━━━━┓");
                println!(
                    "\x1b[0m\x1b[1;31m ┃ \x1b[0m\x1b[1;32m App_{:<9?} \x1b[0m\x1b[1;31m┃",
                    i
                );
                println!("\x1b[0m\x1b[1;33m ┗━━━━━━━━━━━━━━━━┛");
                println!(
                    "\n \x1b[1;32m Name:                {}",
                    temp_data.name.clone().unwrap()
                );

                println!(
                    " \x1b[1;32m Enabled:             {}",
                    temp_data.enabled.unwrap()
                );

                println!(
                    " \x1b[1;32m Total_Size:          {}\n\n",
                    temp_data.total_size.unwrap()
                );
            }
        }
    }
}

pub async fn print_info(attributes: &mut HardwareAttributes) {
    println!("\n\n\n\x1b[1;32m Kernel Attributes");
    println!(
        "\x1b[1;32m     Sentinel: {}",
        attributes.sentinel.clone().unwrap()
    );
    println!(
        "\x1b[1;32m     Version: {}",
        attributes.kernel_version.unwrap()
    );
    println!("\x1b[1;32m KATLV: APP Memory");
    println!(
        "\x1b[1;32m     app_memory_start: {}",
        attributes.app_mem_start.unwrap()
    );
    println!(
        "\x1b[1;32m     app_memory_len: {}",
        attributes.app_mem_len.unwrap()
    );
    println!("\x1b[1;32m KATLV: Kernel Binary");
    println!(
        "\x1b[1;32m     kernel_binary_start: {}",
        attributes.kernel_bin_start.unwrap()
    );
    println!(
        "\x1b[1;32m     kernel_binary_len: {}\n\n",
        attributes.kernel_bin_len.unwrap()
    );
}
