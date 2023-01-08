use super::EncodingFormat;
use anyhow::Result;

pub struct Latin1;

impl EncodingFormat for Latin1 {
    fn into_java(_str: String) -> Result<Vec<u8>> {
        todo!()
    }

    fn from_java(_str: Vec<u8>) -> Result<String> {
        todo!()
    }
}
