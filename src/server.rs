use crate::database::Database;
use crate::message::*;
use crate::pool::ThreadPool;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;

/// The number of workers in the server's thread pool
const WORKERS: usize = 16;

// TODO:
// Implement the `process_message` function. This function should take a `ServerState`, a `Request`,
// and a `TcpStream`. It should process the request and write the response to the stream.
// Processing the request should simply require calling the appropriate function on the database
// and then creating the appropriate response and turning it into bytes which are sent to along
// the stream by calling the `write_all` method.
fn process_message(state: Arc<ServerState>, request: Request, mut stream: TcpStream) {
    match request {
        Request::Publish { doc } => {
            println!("Inside server publish");
            let result = state.database.publish(doc);
            // check for error?
            let response = Response::PublishSuccess(result);
            stream.write_all(&mut response.to_bytes()).unwrap();
        }
        Request::Retrieve { id } => {
            let result = state.database.retrieve(id);
            match result {
                Some(str) => {
                    let response = Response::RetrieveSuccess(str);
                    stream.write_all(&mut response.to_bytes()).unwrap();
                }
                None => {
                    let response = Response::Failure;
                    stream.write_all(&mut response.to_bytes()).unwrap();
                }
            }
        }
        Request::Search { word } => {
            let results = state.database.search(&word);
            let response = Response::SearchSuccess(results);
            stream.write_all(&mut response.to_bytes()).unwrap();
        }
    }
}

/// A struct that contains the state of the server
struct ServerState {
    /// The database that the server uses to store documents
    database: Database,
    /// The thread pool that the server uses to process requests
    pool: ThreadPool,
    /// A flag that indicates whether the server has been stopped
    is_stopped: AtomicBool,
}
impl ServerState {
    fn new() -> Self {
        Self {
            database: Database::new(),
            pool: ThreadPool::new(WORKERS),
            is_stopped: AtomicBool::new(false),
        }
    }
}

pub struct Server {
    state: Arc<ServerState>,
}
impl Server {
    // Create a new server by using the `ServerState::new` function
    pub fn new() -> Self {
        Server {
            state: Arc::new(ServerState::new()),
        }
    }

    // TODO:
    // Spawn a thread that listens for incoming connections on the given port. When a connection is
    // established, add a task to the thread pool that deserializes the request, and processes it
    // using the `process_message` function.
    //
    // To listen for incoming connections, you can use the `std::net::TcpListener::bind` function.
    // To listen on the local address, you can call `TcpListener::bind(("127.0.0.1", port))`. The
    // resulting TcpListener can be used to accept incoming connections by calling the `accept`
    // method in a loop. This method blocks until a new connection is established, and then returns
    // a new TcpStream and the address of the remote peer. You should move this stream into the
    // task that you send to the thread pool.
    //
    // While looping to accept connections, you should also check the `is_stopped` flag in the
    // `ServerState` to see if the server has been stopped. If it has, you should break out of the
    // loop and return.
    fn listen(&self, port: u16) {
        let state = Arc::clone(&self.state);
        let _response = thread::spawn(move || {
            let listener_result = TcpListener::bind(("127.0.0.1", port));
            if listener_result.is_err() {
                return;
            }
            let listener = listener_result.unwrap();
            loop {
                if state.is_stopped.load(Ordering::SeqCst) {
                    return;
                }
                let connection = listener.accept();
                match connection {
                    Ok((mut stream, address)) => {
                        println!("New connection");
                        let request = Request::from_bytes(&mut stream).unwrap();
                        println!("Request: {:?}", request);
                        let copy = Arc::clone(&state);
                        state
                            .pool
                            .execute(move || process_message(copy, request, stream))
                    }
                    Err(_err) => return,
                }
                if state.is_stopped.load(Ordering::SeqCst) {
                    return;
                }
            }
        });
    }

    // This function has already been partially completed for you
    pub fn run(&self, port: u16) {
        // Set up a signal handler to stop the server when Ctrl-C is pressed
        let state = Arc::clone(&self.state);
        match ctrlc::try_set_handler(move || {
            println!("Stopping server...");
            state.is_stopped.store(true, Ordering::SeqCst);
        }) {
            Ok(_) => {}
            Err(ctrlc::Error::MultipleHandlers) => {}
            Err(e) => {
                panic!("Error setting Ctrl-C handler: {}", e);
            }
        }

        // TODO: Call the listen function and then loop (doing nothing) until the server has been stopped
        self.listen(port);
        while !self.state.is_stopped.load(Ordering::SeqCst) {}
    }
    pub fn stop(&self) {
        self.state.is_stopped.store(true, Ordering::SeqCst);
    }
}
