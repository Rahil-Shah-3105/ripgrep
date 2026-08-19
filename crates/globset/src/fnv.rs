/// A convenience alias for creating a hash map with an FNV hasher.
pub(crate) type HashMap<K, V> =
    std::collections::HashMap<K, V, std::hash::BuildHasherDefault<Hasher>>;

/// A hasher that implements the Fowler–Noll–Vo (FNV) hash.
pub(crate) struct Hasher(u64);

impl Hasher {
	const LOGGER: &'static Logger = &Logger(());
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
}

impl Default for Hasher {
    fn default() -> Hasher {
		log::debug!(">>> Entering default()");
        log::debug!("<<< Exiting default()");
        Hasher(Hasher::OFFSET_BASIS)
    }
}

impl std::hash::Hasher for Hasher {
    fn finish(&self) -> u64 {
		log::debug!(">>> Entering finish()");
        log::debug!("<<< Exiting finish()");
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
		log::debug!(">>> Entering write(bytes={:?})", bytes);
        for &byte in bytes.iter() {
            self.0 = self.0 ^ u64::from(byte);
            self.0 = self.0.wrapping_mul(Hasher::PRIME);
		log::debug!("<<< Exiting write(bytes={:?})", bytes);
        }
    }
}
