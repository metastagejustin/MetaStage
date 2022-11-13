use near_sdk::StorageUsage;
/// Currently, we allow three different types of NFTs per Creator, namely
/// Common, Uncommon, Rare
pub const NFT_RANKING: usize = 3;
/// The storage cost of an AccountId type (8 bytes)
pub const ACCOUNT_ID_STORAGE_COST: StorageUsage = 8;
/// The storage cost of creator registry 2 * AccountId + CreatorMetadata (which we allow to be at least 1kb)
pub const CREATOR_REGISTRY_STORAGE_COST: StorageUsage = 1_016;
