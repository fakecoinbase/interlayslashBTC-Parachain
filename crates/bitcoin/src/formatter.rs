use primitive_types::U256;
use sp_std::vec::Vec;
use sp_std::{prelude::*, vec};

use crate::merkle::MerkleProof;
use crate::types::*;
use crate::utils::log256;

const WITNESS_FLAG: u8 = 0x01;
const WITNESS_MARKER: u8 = 0x00;

/// Type to be formatted as a bytes array
pub trait Formattable<Options = ()> {
    fn format(&self) -> Vec<u8>;
    fn format_with(&self, _options: Options) -> Vec<u8> {
        self.format()
    }
}

/// Macro to generate `Formattable` implementation of int types
macro_rules! make_formattable_int {
    ($type:ty) => {
        impl Formattable<bool> for $type {
            fn format(&self) -> Vec<u8> {
                Vec::from(&self.to_le_bytes()[..])
            }

            fn format_with(&self, be: bool) -> Vec<u8> {
                if be {
                    Vec::from(&self.to_be_bytes()[..])
                } else {
                    self.format()
                }
            }
        }
    };
}

impl<T, U> Formattable<U> for &T
where
    T: Formattable<U>,
{
    fn format(&self) -> Vec<u8> {
        T::format(self)
    }

    fn format_with(&self, options: U) -> Vec<u8> {
        T::format_with(self, options)
    }
}

// Generate formattable implementation for the basic integers (signed and unsgined) types
make_formattable_int!(u8);
make_formattable_int!(u16);
make_formattable_int!(u32);
make_formattable_int!(u64);
make_formattable_int!(i8);
make_formattable_int!(i16);
make_formattable_int!(i32);
make_formattable_int!(i64);

impl Formattable<()> for bool {
    fn format(&self) -> Vec<u8> {
        (*self as u8).format()
    }
}

impl Formattable<()> for H256Le {
    fn format(&self) -> Vec<u8> {
        Vec::from(&self.to_bytes_le()[..])
    }
}

impl Formattable for CompactUint {
    fn format(&self) -> Vec<u8> {
        let mut formatter = Formatter::new();
        if self.value < 0xfd {
            formatter.format(self.value as u8);
        } else if self.value < u16::max_value() as u64 {
            formatter.format(0xfd as u8);
            formatter.format(self.value as u16);
        } else if self.value < u32::max_value() as u64 {
            formatter.format(0xfe as u8);
            formatter.format(self.value as u32);
        } else {
            formatter.format(0xff as u8);
            formatter.format(self.value);
        }
        formatter.result()
    }
}

impl<T, U> Formattable<U> for Vec<T>
where
    for<'a> &'a T: Formattable<U>,
    U: Default + Copy,
{
    fn format(&self) -> Vec<u8> {
        self.format_with(Default::default())
    }

    fn format_with(&self, options: U) -> Vec<u8> {
        let mut formatter = Formatter::new();
        formatter.format(CompactUint {
            value: self.len() as u64,
        });
        for value in self.iter() {
            formatter.format_with(value, options);
        }
        formatter.result()
    }
}

impl Formattable<bool> for TransactionInput {
    fn format(&self) -> Vec<u8> {
        let mut formatter = Formatter::new();
        formatter.format(&self.previous_hash);
        formatter.format(self.previous_index);
        formatter.format(CompactUint::from_usize(self.script.len()));
        self.height.iter().for_each(|h| formatter.output(h));
        formatter.output(&self.script); // we already formatted the length
        formatter.format(self.sequence);
        formatter.result()
    }
}

impl Formattable for Script {
    fn format(&self) -> Vec<u8> {
        self.bytes.format()
    }
}

impl Formattable for &[u8] {
    fn format(&self) -> Vec<u8> {
        Vec::from(*self)
    }
}

impl Formattable for Address {
    fn format(&self) -> Vec<u8> {
        Vec::from(self.as_bytes())
    }
}

impl Formattable for OpCode {
    fn format(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl Formattable for TransactionOutput {
    fn format(&self) -> Vec<u8> {
        let mut formatter = Formatter::new();
        formatter.format(self.value);
        formatter.format(&self.script);
        formatter.result()
    }
}

impl Formattable<bool> for Transaction {
    fn format(&self) -> Vec<u8> {
        self.format_with(true)
    }

    fn format_with(&self, witness: bool) -> Vec<u8> {
        let mut formatter = Formatter::new();
        let allow_witness = (self.version & SERIALIZE_TRANSACTION_NO_WITNESS) == 0;
        // check if any of the inputs has a witness
        let has_witness = allow_witness && self.inputs.iter().any(|v| !v.witness.is_empty());

        formatter.format(self.version);

        if witness && has_witness {
            formatter.format(WITNESS_MARKER);
            formatter.format(WITNESS_FLAG);
        }

        formatter.format(&self.inputs);
        formatter.format(&self.outputs);

        if witness && has_witness {
            for input in self.inputs.iter() {
                formatter.format(&input.witness);
            }
        }

        // only block_height or locktime should ever be Some
        self.block_height
            .or(self.locktime)
            .iter()
            .for_each(|b| formatter.format(b));

        formatter.result()
    }
}

impl Formattable<bool> for U256 {
    fn format(&self) -> Vec<u8> {
        let mut bytes: [u8; 4] = Default::default();
        let exponent = log256(&self);
        bytes[3] = exponent;
        let mantissa = if exponent > 3 {
            self / U256::from(256).pow(U256::from(exponent) - 3)
        } else {
            self.clone()
        };
        let mut mantissa_bytes: [u8; 32] = Default::default();
        mantissa.to_little_endian(&mut mantissa_bytes);
        for i in 0..=2 {
            // only three first bytes should be set because of the division
            bytes[i] = mantissa_bytes[i];
        }
        Vec::from(&bytes[..])
    }
}

impl Formattable for BlockHeader {
    fn format(&self) -> Vec<u8> {
        let mut formatter = Formatter::new();
        formatter.format(self.version);
        formatter.format(self.hash_prev_block);
        formatter.format(self.merkle_root);
        formatter.format(self.timestamp);
        formatter.format(self.target);
        formatter.format(self.nonce);
        formatter.result()
    }
}

impl Formattable for Block {
    fn format(&self) -> Vec<u8> {
        let mut formatter = Formatter::new();
        formatter.format(self.header);
        formatter.format(&self.transactions);
        formatter.result()
    }
}

/// Block header (80 bytes)
/// Number of transactions in the block (unsigned int, 4 bytes, little endian)
/// Number of hashes (varint, 1 - 3 bytes)
/// Hashes (N * 32 bytes, little endian)
/// Number of bytes of flag bits (varint, 1 - 3 bytes)
/// Flag bits (little endian)
impl Formattable for MerkleProof {
    fn format(&self) -> Vec<u8> {
        let mut formatter = Formatter::new();
        formatter.format(self.block_header);
        formatter.format(self.transactions_count);
        let hashes_count = CompactUint::from_usize(self.hashes.len());
        formatter.format(hashes_count);
        for hash in self.hashes.clone() {
            formatter.format(hash);
        }

        let mut bytes: Vec<u8> = vec![0; (self.flag_bits.len() + 7) / 8];
        for p in 0..self.flag_bits.len() {
            bytes[p / 8] |= (self.flag_bits[p] as u8) << (p % 8) as u8;
        }
        formatter.format(bytes.len() as u8);
        formatter.output(&bytes);

        formatter.result()
    }
}

pub(crate) struct Formatter {
    bytes: Vec<u8>,
}

impl Formatter {
    fn new() -> Formatter {
        Formatter { bytes: Vec::new() }
    }

    fn output(&mut self, bytes: &Vec<u8>) {
        self.bytes.extend(bytes);
    }

    fn format<T, U>(&mut self, value: T)
    where
        T: Formattable<U>,
    {
        self.bytes.extend(value.format())
    }

    fn format_with<T, U>(&mut self, value: T, data: U)
    where
        T: Formattable<U>,
    {
        self.bytes.extend(value.format_with(data))
    }

    pub(crate) fn result(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::utils::sha256d_le;

    #[test]
    fn test_format_int_types() {
        assert_eq!(1u8.format(), [1]);
        assert_eq!(1i8.format(), [1]);
        assert_eq!(255u8.format(), [255]);
        assert_eq!((-1i8).format(), [255]);

        assert_eq!(256u16.format(), [0, 1]);
        assert_eq!(256u16.format_with(true), [1, 0]);
        assert_eq!((0xffffu32 + 1).format(), [0, 0, 1, 0]);
        assert_eq!((0xffffffu32 + 1).format(), [0, 0, 0, 1]);
        assert_eq!(u64::max_value().format(), [0xff].repeat(8));
    }

    #[test]
    fn test_format_compact_uint() {
        assert_eq!(CompactUint { value: 0xfa }.format(), [0xfa]);
        assert_eq!(CompactUint { value: 0xff }.format(), [0xfd, 0xff, 0]);
        let u32_cuint = CompactUint { value: 0xffff + 1 };
        assert_eq!(u32_cuint.format(), [0xfe, 0, 0, 1, 0]);
        let u64_cuint = CompactUint {
            value: u64::max_value(),
        };
        assert_eq!(u64_cuint.format(), [0xff].repeat(9));
    }

    #[test]
    fn test_format_transaction_input() {
        let raw_input = parser::tests::sample_transaction_input();
        let input_bytes = hex::decode(&raw_input).unwrap();
        let mut parser = parser::BytesParser::new(&input_bytes);
        let input: TransactionInput = parser.parse_with(2).unwrap();
        let formatted = input.format();
        assert_eq!(formatted, input_bytes);
    }

    #[test]
    fn test_format_transaction_output() {
        let raw_output = parser::tests::sample_transaction_output();
        let output_bytes = hex::decode(&raw_output).unwrap();
        let mut parser = parser::BytesParser::new(&output_bytes);
        let output: TransactionOutput = parser.parse().unwrap();
        let formatted = output.format();
        assert_eq!(formatted, output_bytes);
    }

    #[test]
    fn test_format_transaction() {
        let raw_tx = parser::tests::sample_transaction();
        let tx_bytes = hex::decode(&raw_tx).unwrap();
        let transaction = crate::parser::parse_transaction(&tx_bytes).unwrap();
        let formatted = transaction.format();
        assert_eq!(formatted, tx_bytes);
    }

    #[test]
    fn test_format_extended_transaction() {
        let expected_hash =
            H256Le::from_hex_be("b759d39a8596b70b3a46700b83e1edb247e17ba58df305421864fe7a9ac142ea");
        let expected_txid =
            H256Le::from_hex_be("c586389e5e4b3acb9d6c8be1c19ae8ab2795397633176f5a6442a261bbdefc3a");
        let raw_tx = parser::tests::sample_extended_transaction();
        let tx_bytes = hex::decode(&raw_tx).unwrap();
        let transaction = parser::parse_transaction(&tx_bytes).unwrap();
        let formatted = transaction.format();
        assert_eq!(formatted, tx_bytes);
        let computed_hash = sha256d_le(&formatted);
        assert_eq!(computed_hash, expected_hash);
        let formatted_no_witness = transaction.format_with(false);
        let computed_txid = sha256d_le(&formatted_no_witness);
        assert_eq!(computed_txid, expected_txid);
    }

    #[test]
    fn test_format_block_header() {
        let hex_header = parser::tests::sample_block_header();
        let raw_header = RawBlockHeader::from_hex(&hex_header).unwrap();
        let parsed_header = parser::parse_block_header(&raw_header).unwrap();
        assert_eq!(parsed_header.format(), raw_header.as_bytes());
    }

    // taken from https://bitcoin.org/en/developer-reference#block-headers
    #[test]
    fn test_format_u256() {
        let value = U256::from_dec_str("680733321990486529407107157001552378184394215934016880640")
            .unwrap();
        let result = value.format();
        let expected = hex::decode("30c31b18").unwrap();
        assert_eq!(result, expected);
    }

    const PROOF_HEX: &str = "00004020e2ac770a4f511b7ed2f3b638fe12d39ff52b8ced104d360500000000000000006f5ca47842fdd12f46a274ce7060c701d0c1fcff294a826e19b88e8f3dcdbca8f560135e8b64051816587c9c1f0100000bc21da39408e165a8368a7df46a17af25b4c5e3778b45222e48da632412b3be56e3b1196586e514fba3145219e3d9edb1e0e2c71b4cedaf013d8512d121f55e1ae120e954338e4d63d0a446a466b4ec548704366a89c2513c0c47818e4f8af8fa141bcda354451c2a48425704decd178df3c2c518c2fee2a593058b2c2c2ddee80ebc68aa38c161fcbf32f336b9d06feb652893be3326b0fd755cf61e575a56d7cb6b4944a2e74e3fdb583885c9dd4849ab2fd974207d9693a3062d9ba5eb0ea1b7c2d9841297396526c43af19fa8e67f3a6c07f9c8333eda575556df0e8b86a65982f24022336589fae3d56d69d73474024ced4f3a63c7205623d5bd22daf8a58e69b4748539fcdc24e0241f8231278b560340a3eb112f2fd041dc7bd1a0f6ddc37b916c24b0f96a1e9e13b4ffc7ad9c3805cadb91520435821edd439ca70198c92187deb1dde075366006d963632a0fd1ca510b362bbd6cf1805ac70becd3d303ff2d00";

    #[test]
    fn test_format_merkle_proof() {
        let proof = MerkleProof::parse(&hex::decode(&PROOF_HEX[..]).unwrap()).unwrap();
        let expected = hex::decode(PROOF_HEX).unwrap();
        assert_eq!(proof.format(), expected);
    }
}