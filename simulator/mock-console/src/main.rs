//! `mock-console` binary: prints the scenario table. (UDP/FTP servers arrive in phase 3/5.)
fn main() {
    for s in ps4studio_mock_console::scenarios() {
        println!("{:<22} {}", s.id, s.label);
    }
}
