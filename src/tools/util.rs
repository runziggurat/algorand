/// Generate a random data.
pub fn gen_rand_bytes(len: usize) -> Vec<u8> {
    (0..len).map(|_| rand::random::<u8>()).collect()
}

#[cfg(test)]
mod test {
    use super::*;

    const LEN: usize = 512;

    #[tokio::test]
    async fn gen_rand_bytes_length_same() {
        let a = gen_rand_bytes(LEN);
        let b = gen_rand_bytes(LEN);

        assert_eq!(a.len(), b.len());
        assert_ne!(a, b);
    }

    #[tokio::test]
    async fn gen_rand_bytes_length_different() {
        let a = gen_rand_bytes(LEN);
        let b = gen_rand_bytes(LEN + 1);

        assert_ne!(a.len(), b.len());
        assert_ne!(a, b);
    }
}
