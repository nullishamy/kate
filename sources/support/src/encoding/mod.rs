pub mod latin1;
pub mod utf16;
use anyhow::Result;

use self::utf16::Utf16;

#[derive(Debug, Clone)]
pub enum CompactEncoding {
    Latin1,
    Utf16,
}

pub type EncodedString = (CompactEncoding, Vec<u8>);

pub trait EncodingFormat {
    fn into_java(str: String) -> Result<Vec<u8>>;
    fn from_java(str: Vec<u8>) -> Result<String>;
}

/// Decide, based on heuristics about the contained bytes, which encoding format to use to encode the provided string
/// and then return it, alongside the encoded bytes
pub fn encode_string(str: String) -> Result<EncodedString> {
    // TODO: utilize heuristics to store latin1 conditionally

    let encoded = Utf16::into_java(str)?;
    Ok((CompactEncoding::Utf16, encoded))
}

// Decode an encoded string based on its format
pub fn decode_string(str: EncodedString) -> Result<String> {
    let (encoding, data) = str;
    match encoding {
        CompactEncoding::Utf16 => Utf16::from_java(data),
        CompactEncoding::Latin1 => unimplemented!(),
    }
}
