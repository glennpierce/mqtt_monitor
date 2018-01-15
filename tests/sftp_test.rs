extern crate ssh2;

use std::env;
use std::path::{Path};
use std::str;
use std::net::TcpStream;
use ssh2::Session;

#[test]
fn sftp_test() {

    let working_directory = "Test";
    let connection_string = format!("{host}:{port}", host="10.200.9.10", port="22"); 

    println!("SFtp connection string: {}", &connection_string.to_string());

    // Connect to the local SSH server
    let tcp = TcpStream::connect(connection_string.to_string()).unwrap();
    let mut sess = Session::new().unwrap();
    sess.handshake(&tcp).unwrap();

    let privatekey_filepath = env::home_dir().unwrap().join(".ssh/".to_string() + "id_aquatics_rsa");
    sess.userauth_pubkey_file("LLDC", None, privatekey_filepath.as_path(), None).unwrap();

    if !sess.authenticated() {
        println!("{}", "SFtp incorrect credentials".to_string());
    }

    let sftp = sess.sftp().unwrap();

    if sftp.stat(Path::new(&working_directory)).is_err() {
        sftp.mkdir(Path::new(&working_directory), 0o777).unwrap();
        let mut stat = sftp.stat(Path::new(&working_directory)).unwrap();
        stat.perm = Some(0o777);
        sftp.setstat(Path::new(&working_directory), stat).unwrap();
    }
}