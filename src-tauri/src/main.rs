// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Handle `--serve <dist_path> <port>` for the embedded HTTP server.
    if args.len() >= 4 && args[1] == "--serve" {
        actionguard_lib::run_http_server(&args[2], &args[3]);
        return;
    }

    actionguard_lib::run()
}
