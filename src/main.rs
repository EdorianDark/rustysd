mod notification_handler;
mod services;
mod signal_handler;
mod sockets;
mod start_service;
mod unit_parser;
mod units;
mod dbus_wait;

extern crate signal_hook;

#[macro_use]
extern crate log;
extern crate fern;
extern crate threadpool;
extern crate dbus;

use std::path::PathBuf;

fn main() {
    let unit_dirs = vec![PathBuf::from("./test_units")];
    let socket_dir = PathBuf::from("./notifications");

    // initial loading of the units and matching of the various before/after settings
    // also opening all fildescriptors in the socket files
    let (service_table, socket_table) = unit_parser::load_all_units(&unit_dirs).unwrap();

    use std::sync::{Arc, Mutex};
    let service_table = Arc::new(Mutex::new(service_table));
    let socket_table = Arc::new(Mutex::new(socket_table));

    let notification_eventfd =
        nix::sys::eventfd::eventfd(0, nix::sys::eventfd::EfdFlags::EFD_CLOEXEC).unwrap();
    let stdout_eventfd =
        nix::sys::eventfd::eventfd(0, nix::sys::eventfd::EfdFlags::EFD_CLOEXEC).unwrap();
    let stderr_eventfd =
        nix::sys::eventfd::eventfd(0, nix::sys::eventfd::EfdFlags::EFD_CLOEXEC).unwrap();

    let service_table_clone = service_table.clone();
    let service_table_clone2 = service_table.clone();
    let service_table_clone3 = service_table.clone();

    std::thread::spawn(move || {
        notification_handler::handle_all_streams(notification_eventfd, service_table_clone);
    });

    std::thread::spawn(move || {
        notification_handler::handle_all_std_out(stdout_eventfd, service_table_clone2);
    });
    std::thread::spawn(move || {
        notification_handler::handle_all_std_err(stderr_eventfd, service_table_clone3);
    });

    let eventfds = vec![notification_eventfd, stdout_eventfd, stderr_eventfd];

    // parallel startup of all services
    let pid_table = services::run_services(
        service_table.clone(),
        socket_table.clone(),
        socket_dir.clone(),
        eventfds.clone(),
    );

    notification_handler::notify_event_fds(&eventfds);

    // listen on signals from the child processes
    signal_handler::handle_signals(
        service_table.clone(),
        socket_table.clone(),
        pid_table.clone(),
        socket_dir.clone(),
    );
}
