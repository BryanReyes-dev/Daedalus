fn main() {
    if let Err(error) = daedalus_runtime_ipc::serve_stdio() {
        eprintln!("[Daedalus Runtime] IPC server stopped with an error: {error}");
        std::process::exit(1);
    }
}
