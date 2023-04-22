// Rustreexo
use bitcoin_hashes::{hex, sha256, sha512_256, Hash, HashEngine};
use std::{convert::TryFrom, fmt::Display, ops::Deref};

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash, PartialOrd, Ord)]
/// NodeHash is a wrapper around a 32 byte array that represents a hash of a node in the tree.
/// # Example
/// ```
/// use rustreexo::accumulator::types::NodeHash;
/// let hash = NodeHash::new([0; 32]);
/// assert_eq!(hash.to_string().as_str(), "0000000000000000000000000000000000000000000000000000000000000000");
/// ```
pub enum NodeHash {
    Empty,
    Placeholder,
    Some([u8; 32]),
}
impl Deref for NodeHash {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        match self {
            NodeHash::Some(ref inner) => inner,
            _ => &[0; 32],
        }
    }
}
impl Display for NodeHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        if let NodeHash::Some(ref inner) = self {
            let mut s = String::new();
            for byte in inner.iter() {
                s.push_str(&format!("{:02x}", byte));
            }
            write!(f, "{}", s)
        } else {
            write!(f, "empty")
        }
    }
}

impl From<sha512_256::Hash> for NodeHash {
    fn from(hash: sha512_256::Hash) -> Self {
        NodeHash::Some(hash.to_byte_array())
    }
}
impl From<[u8; 32]> for NodeHash {
    fn from(hash: [u8; 32]) -> Self {
        NodeHash::Some(hash)
    }
}
impl From<&[u8; 32]> for NodeHash {
    fn from(hash: &[u8; 32]) -> Self {
        NodeHash::Some(*hash)
    }
}
#[cfg(test)]
impl TryFrom<&str> for NodeHash {
    type Error = hex::Error;
    fn try_from(hash: &str) -> Result<Self, Self::Error> {
        // This implementation is useful for testing, as it allows to create empty hashes
        // from the string of 64 zeros. Without this, it would be impossible to express this
        // hash in the test vectors.
        if hash == "0000000000000000000000000000000000000000000000000000000000000000" {
            return Ok(NodeHash::Empty);
        }
        let hash = hex::FromHex::from_hex(hash)?;
        Ok(NodeHash::Some(hash))
    }
}

#[cfg(not(test))]
impl TryFrom<&str> for NodeHash {
    type Error = hex::Error;
    fn try_from(hash: &str) -> Result<Self, Self::Error> {
        let inner = hex::FromHex::from_hex(hash)?;
        Ok(NodeHash::Some(inner))
    }
}
impl From<&[u8]> for NodeHash {
    fn from(hash: &[u8]) -> Self {
        let mut inner = [0; 32];
        inner.copy_from_slice(hash);
        NodeHash::Some(inner)
    }
}

impl From<sha256::Hash> for NodeHash {
    fn from(hash: sha256::Hash) -> Self {
        NodeHash::Some(hash.to_byte_array())
    }
}

impl NodeHash {
    /// Tells whether this hash is empty. We use empty hashes throughout the code to represent
    /// leaves we want to delete.
    pub fn is_empty(&self) -> bool {
        match self {
            NodeHash::Empty => true,
            _ => false,
        }
    }
    /// Creates a new NodeHash from a 32 byte array.
    /// # Example
    /// ```
    /// use rustreexo::accumulator::types::NodeHash;
    /// let hash = NodeHash::new([0; 32]);
    /// assert_eq!(hash.to_string().as_str(), "0000000000000000000000000000000000000000000000000000000000000000");
    /// ```
    pub fn new(inner: [u8; 32]) -> Self {
        NodeHash::Some(inner)
    }
    /// Creates an empty hash. This is used to represent leaves we want to delete.
    /// # Example
    /// ```
    /// use rustreexo::accumulator::types::NodeHash;
    /// let hash = NodeHash::empty();
    /// assert!(hash.is_empty());
    /// ```
    pub fn empty() -> Self {
        NodeHash::Empty
    }
    /// parent_hash return the merkle parent of the two passed in nodes.
    /// # Example
    /// ```
    /// use rustreexo::accumulator::types::NodeHash;
    /// let left = NodeHash::new([0; 32]);
    /// let right = NodeHash::new([1; 32]);
    /// let parent = NodeHash::parent_hash(&left, &right);
    /// let expected_parent = NodeHash::from_str("34e33ca0c40b7bd33d28932ca9e35170def7309a3bf91ecda5e1ceb067548a12").unwrap();
    /// assert_eq!(parent, expected_parent);
    /// ```
    pub fn parent_hash(left: &NodeHash, right: &NodeHash) -> NodeHash {
        let mut hash = sha512_256::Hash::engine();
        hash.input(&**left);
        hash.input(&**right);
        sha512_256::Hash::from_engine(hash).into()
    }
    /// Creates a NodeHash from a hex string, you can also use the `TryFrom<&str>` trait.
    /// # Example
    /// ```
    /// use rustreexo::accumulator::types::NodeHash;
    /// let hash = NodeHash::from_str("34e33ca0c40b7bd33d28932ca9e35170def7309a3bf91ecda5e1ceb067548a12").unwrap();
    /// assert_eq!(hash.to_string(), "34e33ca0c40b7bd33d28932ca9e35170def7309a3bf91ecda5e1ceb067548a12");
    /// ```
    pub fn from_str(hash: &str) -> Result<Self, hex::Error> {
        Self::try_from(hash)
    }
    /// Returns a arbitrary placeholder hash that is unlikely to collide with any other hash.
    /// We use this while computing roots to destroy. Don't confuse this with an empty hash.
    pub const fn placeholder() -> Self {
        NodeHash::Placeholder
    }
}

#[cfg(test)]
mod test {
    use bitcoin_hashes::{sha256, Hash, HashEngine};

    use super::NodeHash;

    fn hash_from_u8(value: u8) -> NodeHash {
        let mut engine = bitcoin_hashes::sha256::Hash::engine();

        engine.input(&[value]);

        sha256::Hash::from_engine(engine).into()
    }
    #[test]
    fn test_parent_hash() {
        let hash1 = hash_from_u8(0);
        let hash2 = hash_from_u8(1);

        let parent_hash = NodeHash::parent_hash(&hash1, &hash2);
        assert_eq!(
            parent_hash.to_string().as_str(),
            "02242b37d8e851f1e86f46790298c7097df06893d6226b7c1453c213e91717de"
        );
    }
    #[test]
    fn test_hash_from_str() {
        let hash =
            NodeHash::from_str("6e340b9cffb37a989ca544e6bb780a2c78901d3fb33738768511a30617afa01d")
                .unwrap();
        assert_eq!(hash, hash_from_u8(0));
    }
    #[test]
    fn test_empty_hash() {
        // Only relevant for tests
        let hash =
            NodeHash::from_str("0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();
        assert_eq!(hash, NodeHash::empty());
    }
}
