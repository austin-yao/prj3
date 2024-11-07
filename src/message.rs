/// A request from the client to the server
#[derive(Debug, PartialEq)]
pub enum Request {
    /// Add the document `doc` to the archive
    Publish { doc: String },
    /// Search for the word `word` in the archive
    Search { word: String },
    /// Retrieve the document with the index `id` from the archive
    Retrieve { id: usize },
}
impl Request {
    // Convert the request `self` into a byte vector. See the assignment handout for suggestions on
    // how to represent the request as a series of bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::Publish { doc } => {
                let mut bytes = vec![0];
                let length = doc.len();
                bytes.push(length as u8);
                bytes.extend(doc.as_bytes());
                return bytes;
            }
            Self::Search { word } => {
                let mut bytes = vec![1];
                let length = word.len();
                bytes.push(length as u8);
                bytes.extend(word.as_bytes());
                return bytes;
            }
            Self::Retrieve { id } => {
                let mut bytes = vec![2];
                bytes.extend(id.to_be_bytes().iter());
                return bytes;
            }
        }
    }
    // TODO:
    // Read a request from `reader` and return it. Calling `to_bytes` from above and then calling
    // `from_bytes` should return the original request. If the request is invalid, return `None`.
    pub fn from_bytes<R: std::io::Read>(mut reader: R) -> Option<Self> {
        let mut response_type = [0; 1];
        println!("Inside request from bytes");
        let result = reader.read_exact(&mut response_type);
        if result.is_err() {
            return None;
        }

        println!("Request type: {}", response_type[0]);

        match response_type[0] {
            0 => {
                let mut length_buffer = [0; 1];
                let length_result = reader.read_exact(&mut length_buffer);
                if length_result.is_err() {
                    return None;
                }
                let length = length_buffer[0];

                let mut string_buffer = vec![0; length.into()];
                let read_result = reader.read_exact(&mut string_buffer);
                println!("Finished reading to end");
                if read_result.is_err() {
                    return None;
                }

                let ret = Self::Publish {
                    doc: String::from_utf8(string_buffer).unwrap(),
                };

                return Some(ret);
            }
            1 => {
                let mut length_buffer = [0; 1];
                let length_result = reader.read_exact(&mut length_buffer);
                if length_result.is_err() {
                    return None;
                }
                let length = length_buffer[0];

                let mut string_buffer = vec![0; length.into()];
                let read_result = reader.read_to_end(&mut string_buffer);
                if read_result.is_err() {
                    return None;
                }

                let ret = Self::Search {
                    word: String::from_utf8(string_buffer).unwrap(),
                };

                return Some(ret);
            }
            2 => {
                let mut bytes = [0; 8];
                let read_result = reader.read_exact(&mut bytes);
                if read_result.is_err() {
                    return None;
                }

                let id = usize::from_be_bytes(bytes);
                let ret = Self::Retrieve { id: id };

                return Some(ret);
            }
            _ => return None,
        }
    }
}

/// A response from the server to the client
#[derive(Debug, PartialEq)]
pub enum Response {
    /// The document was successfully added to the archive with the given index
    PublishSuccess(usize),
    /// The search for the word was successful, and the indices of the documents containing the
    /// word are returned
    SearchSuccess(Vec<usize>),
    /// The retrieval of the document was successful, and the document is returned
    RetrieveSuccess(String),
    /// The request failed
    Failure,
}
impl Response {
    // TODO:
    // Convert the request `self` into a byte vector. See the assignment handout for suggestions on
    // how to represent the request as a series of bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Self::PublishSuccess(index) => {
                let mut bytes = vec![0];
                bytes.extend(index.to_be_bytes().iter());
                return bytes;
            }
            Self::SearchSuccess(indices) => {
                let mut bytes = vec![1];
                bytes.push(indices.len() as u8);
                for index in indices {
                    bytes.extend(index.to_be_bytes().iter());
                }

                return bytes;
            }
            Self::RetrieveSuccess(doc) => {
                let mut bytes = vec![2];
                let length = doc.len();
                bytes.push(length as u8);
                bytes.extend(doc.as_bytes());
                return bytes;
            }
            Self::Failure => return vec![3],
        }
    }
    // TODO:
    // Read a request from `reader` and return it. Calling `to_bytes` from above and then calling
    // `from_bytes` should return the original request. If the request is invalid, return `None`.
    pub fn from_bytes<R: std::io::Read>(mut reader: R) -> Option<Self> {
        let mut response_type = [0; 1];
        let result = reader.read_exact(&mut response_type);
        if result.is_err() {
            return None;
        }

        match response_type[0] {
            0 => {
                let mut bytes = [0; 8];
                let read_result = reader.read_exact(&mut bytes);
                if read_result.is_err() {
                    return None;
                }

                let id = usize::from_be_bytes(bytes);
                let ret = Self::PublishSuccess(id);

                return Some(ret);
            }
            1 => {
                let mut length_buffer = [0; 1];
                let length_result = reader.read_exact(&mut length_buffer);
                if length_result.is_err() {
                    return None;
                }

                let mut ret_vec: Vec<usize> = Vec::new();
                for _ in 0..length_buffer[0] {
                    let mut bytes = [0; 8];
                    let read_result = reader.read_exact(&mut bytes);
                    if read_result.is_err() {
                        return None;
                    }

                    let id = usize::from_be_bytes(bytes);
                    ret_vec.push(id);
                }

                let ret = Self::SearchSuccess(ret_vec);
                return Some(ret);
            }
            2 => {
                let mut length_buffer = [0; 1];
                let length_result = reader.read_exact(&mut length_buffer);
                if length_result.is_err() {
                    return None;
                }
                let length = length_buffer[0];

                let mut string_buffer = vec![0; length.into()];
                let read_result = reader.read_to_end(&mut string_buffer);
                if read_result.is_err() {
                    return None;
                }

                let ret = Self::RetrieveSuccess(String::from_utf8(string_buffer).unwrap());

                return Some(ret);
            }
            3 => return Some(Self::Failure),
            _ => return None,
        };
    }
}
