use sec1::der::asn1::{BitString, ObjectIdentifier};
use sec1::der::{Decodable, Decoder, Encodable, Result, Sequence};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MetaData {
    pub ec_public_key_id: ObjectIdentifier,
    pub secp256k1_id: ObjectIdentifier,
}

impl MetaData {
    fn new() -> Self {
        Self {
            ec_public_key_id: "1.2.840.10045.2.1".parse::<ObjectIdentifier>().unwrap(),
            secp256k1_id: "1.3.132.0.10".parse::<ObjectIdentifier>().unwrap(),
        }
    }
}

impl<'a> Decodable<'a> for MetaData {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self> {
        decoder.sequence(|decoder| {
            let ec_public_key_id = decoder.decode()?;
            let secp256k1_id = decoder.decode()?;

            Ok(Self {
                ec_public_key_id,
                secp256k1_id,
            })
        })
    }
}

impl<'a> Sequence<'a> for MetaData {
    fn fields<F, T>(&self, field_encoder: F) -> Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> Result<T>,
    {
        field_encoder(&[&self.ec_public_key_id, &self.secp256k1_id])
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Asn1PublicKey<'a> {
    pub meta_data: MetaData,
    pub data: BitString<'a>,
}

impl<'a> Asn1PublicKey<'a> {
    fn new(pk: &'a [u8]) -> Self {
        Self {
            meta_data: MetaData::new(),
            data: BitString::from_bytes(pk).unwrap(),
        }
    }
}

impl<'a> Decodable<'a> for Asn1PublicKey<'a> {
    fn decode(decoder: &mut Decoder<'a>) -> Result<Self> {
        decoder.sequence(|decoder| {
            let meta_data = decoder.decode()?;
            let data = decoder.decode()?;

            Ok(Self { meta_data, data })
        })
    }
}

impl<'a> Sequence<'a> for Asn1PublicKey<'a> {
    fn fields<F, T>(&self, field_encoder: F) -> sec1::der::Result<T>
    where
        F: FnOnce(&[&dyn Encodable]) -> sec1::der::Result<T>,
    {
        field_encoder(&[&self.meta_data, &self.data])
    }
}

pub fn public_key_to_asn1_block(public_key: &[u8]) -> Vec<u8> {
    Asn1PublicKey::new(public_key).to_vec().unwrap()
}

#[cfg(test)]
mod tests {
    use crate::public_key::Asn1PublicKey;
    use sec1::der::Decodable;

    #[test]
    fn test() {
        let pk = vec![1, 2, 3, 4, 5];
        let asn1_pk = super::public_key_to_asn1_block(pk.as_slice());
        assert_eq!(
            pk.as_slice(),
            Asn1PublicKey::from_der(asn1_pk.as_slice())
                .unwrap()
                .data
                .as_bytes()
                .unwrap()
        );
    }
}
