use sha2::{Digest, Sha256};

const HASH_SIZE: usize = 32;

pub type Sha256Hash = [u8; HASH_SIZE];

#[derive(Default)]
pub struct Sha256Algorithm {
    sha256: Sha256,
}

impl Sha256Algorithm {
    pub fn update(&mut self, data: impl AsRef<[u8]>) {
        sha2::Digest::update(&mut self.sha256, data);
    }

    pub fn finish(&self) -> Sha256Hash {
        let array = self.sha256.clone().finalize();
        hash_from_slice(array.as_slice()).unwrap()
    }
}

fn hash_from_slice(data: &[u8]) -> Result<Sha256Hash, String> {
    match data.len() {
        HASH_SIZE => {
            let mut ret = [0_u8; HASH_SIZE];
            ret[..].copy_from_slice(data);
            Ok(ret)
        }
        _ => Err(String::from("wrong hash size")),
    }
}

pub fn get_sha256(data: impl AsRef<[u8]>) -> Sha256Hash {
    let mut algorithm = Sha256Algorithm::default();
    algorithm.update(data);
    algorithm.finish()
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {
        let data: Vec<u8> = vec![1, 2, 3, 4, 5, 6];

        let hash = super::get_sha256(data.clone());
        assert_eq!(hash.len(), 32);
        let value: Vec<u8> = vec![
            113, 146, 56, 92, 60, 6, 5, 222, 85, 187, 148, 118, 206, 29, 144, 116, 129, 144, 236,
            179, 42, 142, 237, 127, 82, 7, 179, 12, 246, 161, 254, 137,
        ];
        assert_eq!(hash, value.as_slice());

        // let mut openssl_sha256 = openssl::sha::Sha256::new();
        // openssl_sha256.update(data.as_slice());
        // let hash2 = openssl_sha256.finish();
        // assert_eq!(hash, hash2);
    }
}
