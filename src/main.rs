use clap::{Parser, Subcommand};
use ngram::client::Client;
use ngram::server::Server;

// TODO:
// Fill out the `Args` struct to parse the command line arguments. You may find clap "subcommands"
// helpful.
/// An archive service allowing publishing and searching of books
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand, Debug)]
enum Mode {
    Client {
        /// Address of the server
        server_address: String,

        /// Port of the server
        server_port: u16,

        #[command(subcommand)]
        command: Command,
    },
    Server {
        /// Port of the server
        server_port: u16,
    },
}

#[derive(Subcommand, Debug)]
enum Command {
    Publish { path: String },
    Search { word: String },
    Retrieve { id: usize },
}

// TODO:
// Inspect the contents of the `args` struct that has been created from the command line arguments
// the user passed. Depending on the arguments, either start a server or make a client and send the
// appropriate request. You may find it helpful to print the request response.
fn main() {
    let args = Args::parse();
    match args.mode {
        Mode::Client {
            server_address,
            server_port,
            command,
        } => {
            let client = Client::new(&server_address, server_port);
            match command {
                Command::Publish { path } => {
                    println!(
                        "Publish. Address: {}, port: {}, path: {}",
                        server_address, server_port, path
                    );
                    let response = client.publish_from_path(&path);
                    match response {
                        Some(r) => println!("{:?}", r),
                        None => println!("none"),
                    }
                }
                Command::Search { word } => {
                    println!(
                        "Search. Address: {}, port: {}, word: {}",
                        server_address, server_port, word
                    );
                    let response = client.search(&word);
                    match response {
                        Some(r) => println!("{:?}", r),
                        None => println!("none"),
                    }
                }
                Command::Retrieve { id } => {
                    println!(
                        "Retrieve. Address: {}, port: {}, id: {}",
                        server_address, server_port, id
                    );
                    let response = client.retrieve(id);
                    match response {
                        Some(r) => println!("{:?}", r),
                        None => println!("none"),
                    }
                }
            }
        }
        Mode::Server { server_port } => {
            println!("Server. port: {}", server_port);
            let server = Server::new();
            server.run(server_port);
        }
    }
}
