use borsh::{io, BorshDeserialize, BorshSerialize};

const ADDRESS_BYTES_LEN: usize = 20;

/// Base ETH Address type
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Address(pub [u8; ADDRESS_BYTES_LEN]);

impl BorshSerialize for Address {
    fn serialize<W: io::Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(self.0.as_slice())
    }
}

impl BorshDeserialize for Address {
    fn deserialize_reader<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let mut buf = [0u8; ADDRESS_BYTES_LEN];
        let maybe_read = reader.read_exact(&mut buf);

        if maybe_read.as_ref().err().map(io::Error::kind) == Some(io::ErrorKind::UnexpectedEof) {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Incorrect length of address",
            ));
        }
        maybe_read?;

        Ok(Self(buf))
    }
}
