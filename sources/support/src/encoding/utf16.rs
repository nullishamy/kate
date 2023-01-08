use super::EncodingFormat;
use anyhow::Result;

pub struct Utf16;

impl EncodingFormat for Utf16 {
    fn into_java(str: String) -> Result<Vec<u8>> {
        // get our standard utf16 repr
        let utf16_bytes: Vec<u16> = str.encode_utf16().collect();

        // now, convert it to a java compatible byte array
        let mut byte_array: Vec<u8> = Vec::new();

        // java always uses big endian. to_be_bytes will convert as needed, depending on platform
        for byte in utf16_bytes {
            let [high, low] = byte.to_be_bytes();
            // in BE, high bytes come first
            byte_array.push(high);
            byte_array.push(low);
        }

        Ok(byte_array)
    }

    fn from_java(data: Vec<u8>) -> Result<String> {
        let mut utf16_bytes = Vec::new();
        let mut i = 0;
        while i < data.len() {
            let bytes = [data[i], data[i + 1]];
            let u16 = u16::from_be_bytes(bytes);
            utf16_bytes.push(u16);
            i += 2;
        }

        Ok(String::from_utf16(&utf16_bytes)?)
    }
}
