use std::{
    fmt::Write,
    io::stdout,
    process::exit,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use crossterm::{
    cursor::MoveTo,
    execute,
    style::Print,
    terminal::{window_size, Clear, ClearType},
};

use libc::EXIT_FAILURE;

use crate::{ascii::get_art, auth::FourWayHandshake, ntlook::get_interface_info_idx, OxideRuntime};

pub fn print_ui(
    oxide: &mut OxideRuntime,
    start_time: Instant,
    framerate: u64,
) -> Result<(), std::io::Error> {
    // Update interface
    match oxide.interface.index {
        Some(index) => match get_interface_info_idx(index) {
            Ok(infos) => oxide.interface = infos,
            Err(e) => {
                let line = get_art(format!("Failed to get interface info: {}.", e).as_str());
                println!("{}", line);
                exit(EXIT_FAILURE);
            }
        },
        None => {
            let line = get_art("Interface index is none.");
            println!("{}", line);
            exit(EXIT_FAILURE);
        }
    }

    /////////// Clear and Print ///////////
    execute!(stdout(), MoveTo(0, 0)).unwrap();
    execute!(stdout(), Clear(ClearType::All)).unwrap();

    let winsize = if let Ok(winsize) = window_size() {
        (winsize.columns, winsize.rows)
    } else {
        (0, 0)
    };

    if winsize.0 < 100 || winsize.1 < 9 {
        let mid = winsize.1 / 2;
        for n in 0..mid {
            execute!(
                stdout(),
                Print(format!("{:^width$}", "", width = winsize.0 as usize))
            )
            .ok();
        }
        execute!(
            stdout(),
            Print(format!(
                "{:^width$}",
                "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓",
                width = winsize.0 as usize
            ))
        )
        .ok();
        execute!(
            stdout(),
            Print(format!(
                "{:^width$}",
                "┃ AngryOxide terminal too small ┃",
                width = winsize.0 as usize
            ))
        )
        .ok();
        execute!(
            stdout(),
            Print(format!(
                "{:^width$}",
                "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛",
                width = winsize.0 as usize
            ))
        )
        .ok();
        return Ok(());
    }

    match oxide.ui_state.menu {
        0 => execute!(
            stdout(),
            Print(access_points_pane(oxide, winsize, start_time, framerate))
        ),
        1 => execute!(
            stdout(),
            Print(clients_pane(oxide, winsize, start_time, framerate))
        ),
        2 => execute!(
            stdout(),
            Print(handshakes_pane(oxide, winsize, start_time, framerate))
        ),
        3 => execute!(
            stdout(),
            Print(messages_pane(oxide, winsize, start_time, framerate))
        ),
        _ => Ok({}),
    }
}

pub fn access_points_pane(
    oxide: &mut OxideRuntime,
    winsize: (u16, u16),
    start_time: Instant,
    framerate: u64,
) -> String {
    let mut output = String::new();
    let width = (winsize.0) as usize;
    let height = winsize.1 as usize;

    // Elapsed Time
    let total_seconds = start_time.elapsed().as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    let time_str = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

    // Status

    let status = format!(
        "{:^15} | {:^10}",
        format!("Frames #: {}", oxide.frame_count),
        format!("Rate: {}/s", framerate),
    );

    let status2 = format!(
        "Sort: {} {} | Errors: {}",
        match oxide.ui_state.ap_sort {
            0 => "Last",
            1 => "RSSI",
            2 => "CH",
            3 => "Clients",
            4 => "4wHS",
            5 => "PMKID",
            _ => "Last",
        },
        match oxide.ui_state.sort_reverse {
            true => "▲",
            false => "▼",
        },
        oxide.error_count,
    );

    // Tabs
    let title = format!("𝘈𝘯𝘨𝘳𝘺𝘖𝘹𝘪𝘥𝘦 | v0.1 | rage 2023 | {}", time_str);
    let tab_toptop = "┏━━━━━━━━━━━━━━━┓ ┏━━━━━━━━━━━┓ ┏━━━━━━━━━━━━━━┓ ┏━━━━━━━━━━━━┓".to_string();
    let tab_center = "┃ Access Points ┃ ┃  Clients  ┃ ┃  Handshakes  ┃ ┃  Messages  ┃".to_string();
    let tab_bottom = "┛               ┗━┻━━━━━━━━━━━┻━┻━━━━━━━━━━━━━━┻━┻━━━━━━━━━━━━┻".to_string();
    let top_diff = width - tab_toptop.chars().count();
    let center_diff = width - tab_center.chars().count();
    let _ = write!(output, "{:^width$}", title);
    let _ = write!(output, "{}{:>top_diff$}", tab_toptop, status);
    let _ = write!(output, "{}{:>center_diff$}", tab_center, status2);
    let _ = write!(output, "{:━<width$}", tab_bottom);

    /////////// Print Access Points ///////////

    let list_height = height - 5;
    write!(
        output,
        "{:<width$}",
        format!(
            "  {:<15} {:<4} {:<5} {:<5} {:<30} {:<10} {:<5} {:<5} {:<5}",
            "MAC Address", "CH", "RSSI", "Last", "SSID", "Clients", "Tx", "4wHS", "PMKID"
        )
    )
    .ok();

    let mut access_points: Vec<_> = oxide.access_points.get_devices().iter().collect();
    match oxide.ui_state.ap_sort {
        0 => access_points.sort_by(|a, b| b.1.last_recv.cmp(&a.1.last_recv)),
        1 => access_points.sort_by(|a, b| {
            b.1.last_signal_strength
                .value
                .cmp(&a.1.last_signal_strength.value)
        }),
        2 => access_points.sort_by(|a, b| b.1.channel.cmp(&a.1.channel)),
        3 => access_points.sort_by(|a, b| b.1.client_list.size().cmp(&a.1.client_list.size())),
        4 => access_points.sort_by(|a, b| b.1.has_hs.cmp(&a.1.has_hs)),
        5 => access_points.sort_by(|a, b| b.1.has_pmkid.cmp(&a.1.has_pmkid)),
        _ => {
            access_points.sort_by(|a, b| b.1.last_recv.cmp(&a.1.last_recv));
        }
    }

    if oxide.ui_state.sort_reverse {
        access_points.reverse();
    }

    let mut ap_len = 1;
    for (mac, ap_data) in access_points.clone() {
        if ap_len < list_height - 2 {
            let unknown = "Unknown SSID".to_string();
            let mut ssid = ap_data.ssid.clone().unwrap_or(unknown);
            if ssid.is_empty() {
                ssid = "Hidden SSID".to_string()
            }
            if ssid.chars().count() > 30 {
                ssid.truncate(27);
                ssid += "...";
            }
            let clients_size = ap_data.client_list.clone().size();
            let chan = if ap_data.channel.is_some() {
                ap_data.clone().channel.unwrap().short_string()
            } else {
                "?".to_string()
            };
            let hss = oxide.handshake_storage.find_handshakes_by_ap(mac);
            let mut pwnd_counter = 0;
            let mut pmkid_counter = 0;
            for (_, hs_list) in hss {
                for fwhs in hs_list {
                    if fwhs.complete() {
                        pwnd_counter += 1;
                    }
                    if fwhs.has_pmkid() {
                        pmkid_counter += 1;
                    }
                }
            }
            write!(
                output,
                "{:<width$}",
                format!(
                    "  {:<15} {:<4} {:<5} {:<5} {:<30} {:<10} {:<5} {:<5} {:<5}",
                    mac.to_string(),
                    chan,
                    ap_data.last_signal_strength.value.to_string(),
                    epoch_to_string(ap_data.last_recv).to_string(),
                    ssid,
                    clients_size,
                    ap_data.interactions,
                    if pwnd_counter > 0 {
                        "\u{2705}\0".to_string()
                    } else {
                        " ".to_string()
                    },
                    if pmkid_counter > 0 {
                        "\u{2705}\0".to_string()
                    } else {
                        " ".to_string()
                    },
                )
            )
            .ok();
            ap_len += 1;
        } else {
            write!(
                output,
                "{:^width$}",
                format!("---- +{} more ----", access_points.len() - ap_len + 1)
            )
            .ok();
            break;
        }
    }

    let display_height = output.chars().count() / width;
    for _ in display_height..height - 2 {
        write!(output, "{:width$}", "").ok();
    }

    write!(output, "{:─>width$}", "").ok();
    write!(
        output,
        "{:^width$}",
        "quit: [q] | sort APs: [a] | sort clients: [c] | reverse sort: [r] | change tab: [←]/[→]"
    )
    .ok();

    output
}

pub fn clients_pane(
    oxide: &mut OxideRuntime,
    winsize: (u16, u16),
    start_time: Instant,
    framerate: u64,
) -> String {
    let mut output = String::new();
    let width = winsize.0 as usize;
    let height = winsize.1 as usize;

    // Elapsed Time
    let total_seconds = start_time.elapsed().as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    let time_str = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

    // Status
    let status = format!(
        "{:^15} | {:^10}",
        format!("Frames #: {}", oxide.frame_count),
        format!("Rate: {}/s", framerate),
    );

    let status2 = format!(
        "Sort: {} {} | {}",
        match oxide.ui_state.cl_sort {
            0 => "Last",
            1 => "RSSI",
            _ => "Last",
        },
        match oxide.ui_state.sort_reverse {
            true => "▲",
            false => "▼",
        },
        format!("Errors: {}", oxide.error_count),
    );

    // Tabs
    let title = format!("𝘈𝘯𝘨𝘳𝘺𝘖𝘹𝘪𝘥𝘦 | v0.1 | rage 2023 | {}", time_str);
    let tab_toptop = "┏━━━━━━━━━━━━━━━┓ ┏━━━━━━━━━━━┓ ┏━━━━━━━━━━━━━━┓ ┏━━━━━━━━━━━━┓".to_string();
    let tab_center = "┃ Access Points ┃ ┃  Clients  ┃ ┃  Handshakes  ┃ ┃  Messages  ┃".to_string();
    let tab_bottom = "┻━━━━━━━━━━━━━━━┻━┛           ┗━┻━━━━━━━━━━━━━━┻━┻━━━━━━━━━━━━┻".to_string();
    let top_diff = width - tab_toptop.chars().count();
    let center_diff = width - tab_center.chars().count();
    let _ = write!(output, "{:^width$}", title);
    let _ = write!(output, "{}{:>top_diff$}", tab_toptop, status);
    let _ = write!(output, "{}{:>center_diff$}", tab_center, status2);
    let _ = write!(output, "{:━<width$}", tab_bottom);

    let list_height = height - 5;

    let mut client_devices: Vec<_> = oxide.unassoc_clients.get_devices().iter().collect();
    let binding = oxide.access_points.get_all_clients();
    let new_clients: Vec<_> = binding.iter().collect();
    client_devices.extend(new_clients);

    write!(
        output,
        "{:<width$}",
        format!(
            "  {:<15} {:<15} {:<8} {:<6} {:<30}",
            "MAC Address", "Access Point", "RSSI", "Last", "Probes"
        )
    )
    .ok();
    match oxide.ui_state.cl_sort {
        0 => client_devices.sort_by(|a, b| b.1.last_recv.cmp(&a.1.last_recv)),
        1 => client_devices.sort_by(|a, b| {
            b.1.last_signal_strength
                .value
                .cmp(&a.1.last_signal_strength.value)
        }),
        _ => client_devices.sort_by(|a, b| b.1.last_recv.cmp(&a.1.last_recv)),
    }
    if oxide.ui_state.sort_reverse {
        client_devices.reverse();
    }
    let mut client_len = 1;
    for (mac, station_data) in client_devices.clone() {
        if client_len < list_height - 2 {
            let ap = if let Some(access_point) = station_data.access_point {
                access_point.to_string()
            } else {
                "".to_string()
            };

            let mut line = format!(
                "  {:<15} {:<15} {:<8} {:<6} {:<30}",
                mac.to_string(),
                ap,
                if station_data.last_signal_strength.value != 0 {
                    station_data.last_signal_strength.value.to_string()
                } else {
                    "".to_string()
                },
                epoch_to_string(station_data.last_recv),
                station_data.clone().probes_to_string_list(),
            );
            if line.chars().count() > width {
                line.truncate(width - 3);
                line = line + "...";
            }
            write!(output, "{:<width$}", line).ok();
            client_len += 1;
        } else {
            write!(
                output,
                "{:^width$}",
                format!("---- +{} more ----", client_devices.len() - client_len + 1)
            )
            .ok();
            client_len += 1;
            break;
        }
    }
    let display_height = output.chars().count() / width;
    for n in display_height..height - 2 {
        write!(output, "{:width$}", "").ok();
    }

    write!(output, "{:─>width$}", "").ok();
    write!(
        output,
        "{:^width$}",
        "quit: [q] | sort APs: [a] | sort clients: [c] | reverse sort: [r] | change tab: [←]/[→]"
    )
    .ok();

    output
}

pub fn handshakes_pane(
    oxide: &mut OxideRuntime,
    winsize: (u16, u16),
    start_time: Instant,
    framerate: u64,
) -> String {
    let mut output = String::new();
    let width = winsize.0 as usize;
    let height = winsize.1 as usize;

    // Elapsed Time
    let total_seconds = start_time.elapsed().as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    let time_str = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

    // Status
    let status = format!(
        "{:^15} | {:^10}",
        format!("Frames #: {}", oxide.frame_count),
        format!("Rate: {}/s", framerate),
    );

    let status2 = format!("{}", format!("Errors: {}", oxide.error_count),);

    /// Tabs
    let title = format!("𝘈𝘯𝘨𝘳𝘺𝘖𝘹𝘪𝘥𝘦 | v0.1 | rage 2023 | {}", time_str);
    let tab_toptop = "┏━━━━━━━━━━━━━━━┓ ┏━━━━━━━━━━━┓ ┏━━━━━━━━━━━━━━┓ ┏━━━━━━━━━━━━┓".to_string();
    let tab_center = "┃ Access Points ┃ ┃  Clients  ┃ ┃  Handshakes  ┃ ┃  Messages  ┃".to_string();
    let tab_bottom = "┻━━━━━━━━━━━━━━━┻━┻━━━━━━━━━━━┻━┛              ┗━┻━━━━━━━━━━━━┻".to_string();
    let top_diff = width - tab_toptop.chars().count();
    let center_diff = width - tab_center.chars().count();
    let _ = write!(output, "{:^width$}", title);
    let _ = write!(output, "{}{:>top_diff$}", tab_toptop, status);
    let _ = write!(output, "{}{:>center_diff$}", tab_center, status2);
    let _ = write!(output, "{:━<width$}", tab_bottom);

    /// List
    let list_height = height - 5;

    let headers = [
        "AP MAC",
        "Client MAC",
        "SSID",
        "[M1 M2 M3 M4 MC] | [PM] | COMPLETE",
    ];
    write!(
        output,
        "{:<width$}",
        format!(
            "  {:<15} {:<15} {:<30} {:<30}",
            headers[0], headers[1], headers[2], headers[3],
        )
    )
    .ok();

    let mut print_handshakes: Vec<&FourWayHandshake> = Vec::new();
    let mut hs_len = 0;
    let binding = oxide.handshake_storage.get_handshakes();
    for handshake_list in binding.values() {
        for handshake in handshake_list {
            print_handshakes.push(handshake);
        }
    }

    print_handshakes.sort_by(|a, b| {
        b.last_msg
            .clone()
            .unwrap()
            .timestamp
            .cmp(&a.last_msg.clone().unwrap().timestamp)
    });
    for hs in print_handshakes {
        let mut ssid = hs.essid_to_string();
        if ssid.chars().count() > 30 {
            ssid.truncate(27);
            ssid += "...";
        }
        write!(
            output,
            "{:<width$}",
            format!(
                "  {:<15} {:<15} {:<30} {:<30}",
                hs.mac_ap.unwrap().to_string(),
                hs.mac_client.unwrap().to_string(),
                ssid,
                hs.to_string()
            )
        )
        .ok();
        hs_len += 1;
        if hs_len >= list_height - 2 {
            if oxide.handshake_storage.count() > 6 {
                write!(
                    output,
                    "{:^width$}",
                    format!(
                        "---- +{} more ----",
                        oxide.handshake_storage.count() - hs_len + 1
                    )
                )
                .ok();
            }
            break;
        }
    }
    let display_height = output.chars().count() / width;
    for n in display_height..height - 2 {
        write!(output, "{:width$}", "").ok();
    }

    write!(output, "{:─>width$}", "").ok();
    write!(
        output,
        "{:^width$}",
        "quit: [q] | sort APs: [a] | sort clients: [c] | reverse sort: [r] | change tab: [←]/[→]"
    )
    .ok();

    output
}

pub fn messages_pane(
    oxide: &mut OxideRuntime,
    winsize: (u16, u16),
    start_time: Instant,
    framerate: u64,
) -> String {
    let mut output = String::new();
    let width = winsize.0 as usize;
    let height = winsize.1 as usize;

    // Elapsed Time
    let total_seconds = start_time.elapsed().as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    let time_str = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

    // Status
    let status = format!(
        "{:^15} | {:^10}",
        format!("Frames #: {}", oxide.frame_count),
        format!("Rate: {}/s", framerate),
    );

    let status2 = format!("{}", format!("Errors: {}", oxide.error_count),);

    /// Tabs
    let title = format!("𝘈𝘯𝘨𝘳𝘺𝘖𝘹𝘪𝘥𝘦 | v0.1 | rage 2023 | {}", time_str);
    let tab_toptop = "┏━━━━━━━━━━━━━━━┓ ┏━━━━━━━━━━━┓ ┏━━━━━━━━━━━━━━┓ ┏━━━━━━━━━━━━┓".to_string();
    let tab_center = "┃ Access Points ┃ ┃  Clients  ┃ ┃  Handshakes  ┃ ┃  Messages  ┃".to_string();
    let tab_bottom = "┻━━━━━━━━━━━━━━━┻━┻━━━━━━━━━━━┻━┻━━━━━━━━━━━━━━┻━┛            ┗".to_string();
    let top_diff = width - tab_toptop.chars().count();
    let center_diff = width - tab_center.chars().count();
    let _ = write!(output, "{:^width$}", title);
    let _ = write!(output, "{}{:>top_diff$}", tab_toptop, status);
    let _ = write!(output, "{}{:>center_diff$}", tab_center, status2);
    let _ = write!(output, "{:━<width$}", tab_bottom);

    write!(
        output,
        "{:<width$}",
        format!("  {:<25} {:<6} {}", "Date / Time", "Type", "Message")
    )
    .ok();
    let list_height = height - 5;

    let mut recent_messages = oxide.status_log.get_recent_messages(list_height - 2);
    recent_messages.reverse();
    for message in recent_messages {
        let mut line = format!(
            "  {:<25} {:<6} {}",
            message.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            message.message_type.to_string(),
            message.content.to_string()
        );
        if line.chars().count() > width {
            line.truncate(width - 3);
            line = line + "...";
        }

        write!(output, "{:<width$}", line).ok();
    }
    let display_height = output.chars().count() / width;
    for n in display_height..height - 2 {
        write!(output, "{:width$}", "").ok();
    }

    write!(output, "{:─>width$}", "").ok();
    write!(
        output,
        "{:^width$}",
        "quit: [q] | sort APs: [a] | sort clients: [c] | reverse sort: [r] | change tab: [←]/[→]"
    )
    .ok();

    output
}

pub fn default_ui(oxide: &mut OxideRuntime, start_time: Instant, framerate: u64) -> String {
    // Update interface
    match oxide.interface.index {
        Some(index) => match get_interface_info_idx(index) {
            Ok(infos) => oxide.interface = infos,
            Err(e) => {
                eprintln!("Failed to get interface info: {}", e);
                exit(EXIT_FAILURE);
            }
        },
        None => {
            eprintln!("Interface index is None");
            exit(EXIT_FAILURE);
        }
    }

    let mut output = String::new();

    /////////// Print Status Bar ///////////

    // Elapsed Time
    let total_seconds = start_time.elapsed().as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    let time_str = format!("{:02}:{:02}:{:02}", hours, minutes, seconds);

    let status = format!(
        "{:^15} | {:^15} | {:^10} | {:^10} | {:^10}",
        format!(
            "Channel: {}",
            oxide
                .interface
                .frequency
                .as_ref()
                .unwrap()
                .channel
                .as_ref()
                .map_or("None".to_string(), |value| value.to_string())
        ),
        format!("Frames #: {}", oxide.frame_count),
        format!("Rate: {}/s", framerate),
        format!("Mac: {}", oxide.rogue_client),
        format!("Errors: {}", oxide.error_count),
    );
    writeln!(output, "{:<7} {:>7} | {:>80}", "WPOxide", time_str, status,).ok();
    writeln!(output, "{}", "-".repeat(101)).ok();

    /////////// Print Access Points ///////////

    let aps = format!("Access Points: {}", oxide.access_points.size());
    writeln!(
        output,
        "{} {} {}",
        "=".repeat((100 - aps.len()) / 2),
        aps,
        "=".repeat((100 - aps.len()) / 2)
    )
    .ok();

    writeln!(
        output,
        "{:<15} {:<4} {:<5} {:<5} {:<30} {:<10} {:<5} {:<5} {:<5}",
        "MAC Address", "CH", "RSSI", "Last", "SSID", "Clients", "Int.", "4wHS", "PMKID",
    )
    .ok();

    let mut access_points: Vec<_> = oxide.access_points.get_devices().iter().collect();
    access_points.sort_by(|a, b| b.1.last_recv.cmp(&a.1.last_recv));

    let mut ap_len = 1;
    for (mac, ap_data) in access_points.clone() {
        if ap_len < 19 {
            let unknown = "Unknown SSID".to_string();
            let mut ssid = ap_data.ssid.clone().unwrap_or(unknown);
            if ssid == " " {
                ssid = "Hidden SSID".to_string()
            }
            let clients_size = ap_data.client_list.clone().size();
            let chan = if ap_data.channel.is_some() {
                ap_data.clone().channel.unwrap().short_string()
            } else {
                "?".to_string()
            };
            let hss = oxide.handshake_storage.find_handshakes_by_ap(mac);
            let mut pwnd_counter = 0;
            let mut pmkid_counter = 0;
            for (_, hs_list) in hss {
                for fwhs in hs_list {
                    if fwhs.complete() {
                        pwnd_counter += 1;
                    }
                    if fwhs.has_pmkid() {
                        pmkid_counter += 1;
                    }
                }
            }
            writeln!(
                output,
                "{:<15} {:<4} {:<5} {:<5} {:<30} {:<10} {:<5} {:<5} {:<5}",
                mac.to_string(),
                chan,
                ap_data.last_signal_strength.value.to_string(),
                epoch_to_string(ap_data.last_recv).to_string(),
                ssid,
                clients_size,
                ap_data.interactions,
                if pwnd_counter > 0 {
                    '\u{2705}'.to_string()
                } else {
                    " ".to_string()
                },
                if pmkid_counter > 0 {
                    '\u{2705}'.to_string()
                } else {
                    " ".to_string()
                },
            )
            .ok();
            ap_len += 1;
        } else {
            writeln!(
                output,
                "{:^100}",
                format!("---- +{} more ----", access_points.len() - ap_len)
            )
            .ok();
            break;
        }
    }
    for _ in 0..(20 - ap_len) {
        writeln!(output).ok();
    }

    /////////// Print Clients ///////////

    let mut client_devices: Vec<_> = oxide.unassoc_clients.get_devices().iter().collect();
    let binding = oxide.access_points.get_all_clients();
    let new_clients: Vec<_> = binding.iter().collect();
    client_devices.extend(new_clients);

    let clnt = format!("Clients: {}", client_devices.len());
    writeln!(
        output,
        "{} {} {}",
        "=".repeat((100 - clnt.len()) / 2),
        clnt,
        "=".repeat((100 - clnt.len()) / 2)
    )
    .ok();

    writeln!(
        output,
        "{:<15} {:<15} {:<8} {:<18} {:<40}",
        "MAC Address", "Access Point", "RSSI", "Last Seen", "Probes"
    )
    .ok();

    client_devices.sort_by(|a, b| b.1.last_recv.cmp(&a.1.last_recv));
    let mut client_len = 0;
    for (mac, station_data) in client_devices.clone() {
        if client_len < 15 {
            let ap = if let Some(access_point) = station_data.access_point {
                access_point.to_string()
            } else {
                "".to_string()
            };
            writeln!(
                output,
                "{:<15} {:<15} {:<8} {:<18} {:<40}",
                mac.to_string(),
                ap,
                if station_data.last_signal_strength.value != 0 {
                    station_data.last_signal_strength.value.to_string()
                } else {
                    "".to_string()
                },
                epoch_to_string(station_data.last_recv),
                station_data.clone().probes_to_string_list(),
            )
            .ok();
            client_len += 1;
        } else {
            writeln!(
                output,
                "{:^100}",
                format!("---- +{} more ----", client_devices.len() - client_len)
            )
            .ok();
            client_len += 1;
            break;
        }
    }
    for _ in 0..(17 - client_len) {
        writeln!(output).ok();
    }
    writeln!(output, "{}", "-".repeat(101)).ok();

    /////////// Print Handshakes ///////////

    let clnt = format!("Handshakes: {}", oxide.handshake_storage.count());
    writeln!(
        output,
        "{} {} {}",
        "=".repeat((100 - clnt.len()) / 2),
        clnt,
        "=".repeat((100 - clnt.len()) / 2)
    )
    .ok();

    let headers = [
        "AP MAC",
        "Client MAC",
        "ESSID",
        "[M1 M2 M3 M4 MC] | [PM] | COMPLETE",
    ];
    writeln!(
        output,
        "{:<15} {:<15} {:<30} {:<30}",
        headers[0], headers[1], headers[2], headers[3],
    )
    .ok();

    let mut print_handshakes: Vec<&FourWayHandshake> = Vec::new();
    let mut hs_len = 0;
    let binding = oxide.handshake_storage.get_handshakes();
    for handshake_list in binding.values() {
        for handshake in handshake_list {
            print_handshakes.push(handshake);
        }
    }

    print_handshakes.sort_by(|a, b| {
        b.last_msg
            .clone()
            .unwrap()
            .timestamp
            .cmp(&a.last_msg.clone().unwrap().timestamp)
    });
    for hs in print_handshakes {
        writeln!(
            output,
            "{:<15} {:<15} {:<30} {:<30}",
            hs.mac_ap.unwrap().to_string(),
            hs.mac_client.unwrap().to_string(),
            hs.essid_to_string(),
            hs.to_string()
        )
        .ok();
        hs_len += 1;
        if hs_len >= 6 {
            if oxide.handshake_storage.count() > 6 {
                writeln!(
                    output,
                    "{:^100}",
                    format!(
                        "---- +{} more ----",
                        oxide.handshake_storage.count() - hs_len
                    )
                )
                .ok();
            }
            break;
        }
    }

    for _ in 0..(7 - hs_len) {
        writeln!(output).ok();
    }

    /////////// Print Status Messages ///////////

    writeln!(
        output,
        "{} Messages {}=",
        "=".repeat((99 - "Messages".len()) / 2),
        "=".repeat((99 - "Messages".len()) / 2)
    )
    .ok();
    let mut recent_messages = oxide.status_log.get_recent_messages(15);
    recent_messages.reverse();
    for message in recent_messages {
        writeln!(
            output,
            "{}: ({}) {}",
            message.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            message.message_type,
            message.content
        )
        .ok();
    }

    output
}

fn epoch_to_string(epoch: u64) -> String {
    match UNIX_EPOCH.checked_add(Duration::from_secs(epoch)) {
        Some(epoch_time) => match SystemTime::now().duration_since(epoch_time) {
            Ok(duration_since) => {
                let elapsed_seconds = duration_since.as_secs();
                if elapsed_seconds > 3600 {
                    format!("{}h", elapsed_seconds / 3600)
                } else if duration_since.as_secs() > 60 {
                    format!("{}m", elapsed_seconds / 60)
                } else {
                    format!("{}s", elapsed_seconds)
                }
            }
            Err(_) => "Time is in the future".to_string(),
        },
        None => "Invalid timestamp".to_string(),
    }
}
