use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce
};
use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use hex;
use std::error::Error;

const BLOCK_SIZE: usize = 16; // 字节

/// 加密元数据，用于记录拆分、换序、填充的信息
#[derive(Serialize, Deserialize, Debug)]
struct Metadata {
    /// 原始密文长度（AES加密后的长度）
    original_len: usize,
    /// 真实块的排列顺序，每个元素是原始块索引
    permutation: Vec<usize>,
    /// 填充块的位置（在最终块列表中的索引）
    padding_positions: Vec<usize>,
}

/// 加密服务，持有私钥并提供加密/解密功能
#[derive(Clone)]
pub struct CryptoService {
    /// 私钥，32字节，用于AES-256-GCM加密
    private_key: [u8; 32],
}

impl CryptoService {
    /// 生成一个新的随机私钥
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        CryptoService {
            private_key: key,
        }
    }

    /// 从已有的私钥字节构造（例如从环境变量或配置文件读取）
    pub fn from_key(private_key: [u8; 32]) -> Self {
        CryptoService {
            private_key,
        }
    }

    /// 从十六进制字符串加载私钥
    pub fn from_hex(hex_key: &str) -> Result<Self, Box<dyn Error>> {
        let bytes = hex::decode(hex_key)?;
        if bytes.len() != 32 {
            return Err("私钥长度必须为32字节".into());
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(&bytes);
        Ok(Self::from_key(key))
    }

    /// 获取私钥的十六进制表示（用于存储或调试）
    pub fn private_key_hex(&self) -> String {
        hex::encode(self.private_key)
    }

    /// 使用私钥派生一个确定性RNG
    fn derive_rng(&self) -> ChaCha20Rng {
        let mut hasher = Sha256::new();
        hasher.update(b"deterministic-rng-seed");
        hasher.update(&self.private_key);
        let seed: [u8; 32] = hasher.finalize().into();
        ChaCha20Rng::from_seed(seed)
    }

    /// 派生密钥流（用于私有算法）
    fn derive_keystream(&self, length: usize) -> Vec<u8> {
        let mut keystream = Vec::with_capacity(length);
        let mut hasher = Sha256::new();
        hasher.update(b"private-algorithm-keystream");
        hasher.update(&self.private_key);
        let mut seed = hasher.finalize();
        while keystream.len() < length {
            keystream.extend_from_slice(&seed);
            let mut hasher = Sha256::new();
            hasher.update(&seed);
            seed = hasher.finalize();
        }
        keystream.truncate(length);
        keystream
    }

    /// 私有算法加密（XOR混淆）
    fn private_encrypt(&self, data: &[u8]) -> Vec<u8> {
        let keystream = self.derive_keystream(data.len());
        data.iter().zip(keystream).map(|(d, k)| d ^ k).collect()
    }

    /// 私有算法解密（XOR混淆，与加密相同）
    fn private_decrypt(&self, data: &[u8]) -> Vec<u8> {
        self.private_encrypt(data)
    }

    /// 核心加密函数：拆分、换序、填充，然后使用AES-GCM加密
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        // 1. 使用私有算法加密原始数据
        let private_encrypted = self.private_encrypt(plaintext);
        // 2. 使用AES-GCM加密私有加密后的数据
        let ciphertext = self.aes_encrypt(&private_encrypted)?;
        
        // 2. 拆分：将密文分成固定大小的块（16字节）
        let blocks = Self::split_into_blocks(&ciphertext, BLOCK_SIZE);
        let original_len = ciphertext.len();
        
        // 3. 换序：使用确定性RNG生成排列顺序
        let mut rng = self.derive_rng();
        let permutation = Self::generate_permutation(blocks.len(), &mut rng);
        let shuffled = Self::apply_permutation(&blocks, &permutation);
        
        // 4. 填充：插入随机块以增加混淆
        let (padded_blocks, padding_positions) = Self::add_padding(shuffled, &mut rng);
        
        // 5. 构建元数据
        let metadata = Metadata {
            original_len,
            permutation,
            padding_positions,
        };
        
        // 6. 加密元数据
        let metadata_bytes = serde_json::to_vec(&metadata)?;
        let encrypted_metadata = self.aes_encrypt(&metadata_bytes)?;
        
        // 7. 合并：加密的元数据长度（2字节） + 加密的元数据 + 混淆后的块数据
        let mut result = Vec::new();
        if encrypted_metadata.len() > 65535 {
            return Err("元数据过长".into());
        }
        result.push((encrypted_metadata.len() >> 8) as u8);
        result.push((encrypted_metadata.len() & 0xFF) as u8);
        result.extend_from_slice(&encrypted_metadata);
        for block in padded_blocks {
            result.extend_from_slice(&block);
        }
        
        Ok(result)
    }

    /// 核心解密函数：逆向操作
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        // 1. 提取元数据长度
        if data.len() < 2 {
            return Err("数据过短".into());
        }
        let metadata_len = ((data[0] as usize) << 8) | (data[1] as usize);
        let metadata_start = 2;
        let metadata_end = metadata_start + metadata_len;
        if metadata_end > data.len() {
            return Err("元数据长度超出数据范围".into());
        }
        let encrypted_metadata = &data[metadata_start..metadata_end];
        let blocks_data = &data[metadata_end..];
        
        // 2. 解密元数据
        let metadata_bytes = self.aes_decrypt(encrypted_metadata)?;
        let metadata: Metadata = serde_json::from_slice(&metadata_bytes)?;
        
        // 3. 将块数据分割成块
        let block_count = blocks_data.len() / BLOCK_SIZE;
        if block_count * BLOCK_SIZE != blocks_data.len() {
            return Err("块数据长度不是块大小的整数倍".into());
        }
        let mut blocks = Vec::with_capacity(block_count);
        for i in 0..block_count {
            let start = i * BLOCK_SIZE;
            let end = start + BLOCK_SIZE;
            blocks.push(blocks_data[start..end].to_vec());
        }
        
        // 4. 移除填充块
        let unpadded = Self::remove_padding(blocks, &metadata.padding_positions)?;
        
        // 5. 恢复原始顺序
        let unshuffled = Self::reverse_permutation(unpadded, &metadata.permutation)?;
        
        // 6. 合并块并截断到原始长度
        let mut ciphertext = Self::merge_blocks(unshuffled, BLOCK_SIZE);
        if ciphertext.len() < metadata.original_len {
            return Err("密文长度不足".into());
        }
        ciphertext.truncate(metadata.original_len);
        
        // 7. 解密AES密文得到明文
        let aes_decrypted = self.aes_decrypt(&ciphertext)?;
        // 8. 使用私有算法解密得到原始明文
        Ok(self.private_decrypt(&aes_decrypted))
    }

    /// AES-GCM加密（使用私钥作为密钥）
    fn aes_encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let key = Key::<Aes256Gcm>::from_slice(&self.private_key);
        let cipher = Aes256Gcm::new(key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 随机nonce
        let ciphertext = cipher.encrypt(&nonce, plaintext)
            .map_err(|e| format!("AES加密失败: {}", e))?;
        // 将nonce和密文拼接（nonce长度固定为12字节）
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(nonce.as_slice());
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// AES-GCM解密
    fn aes_decrypt(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        if data.len() < 12 {
            return Err("数据过短，缺少nonce".into());
        }
        let (nonce_slice, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_slice);
        let key = Key::<Aes256Gcm>::from_slice(&self.private_key);
        let cipher = Aes256Gcm::new(key);
        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| format!("AES解密失败: {}", e))?;
        Ok(plaintext)
    }

    /// 将字节切片拆分成固定大小的块，最后一块不足则补零
    fn split_into_blocks(data: &[u8], block_size: usize) -> Vec<Vec<u8>> {
        let mut blocks = Vec::new();
        for chunk in data.chunks(block_size) {
            let mut block = chunk.to_vec();
            if block.len() < block_size {
                block.resize(block_size, 0); // 补零填充
            }
            blocks.push(block);
        }
        blocks
    }

    /// 合并块，并去除尾部可能多余的零（需要原始长度信息）
    fn merge_blocks(blocks: Vec<Vec<u8>>, block_size: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(blocks.len() * block_size);
        for block in blocks {
            result.extend_from_slice(&block);
        }
        result
    }

    /// 生成一个随机排列
    fn generate_permutation(len: usize, rng: &mut ChaCha20Rng) -> Vec<usize> {
        let mut perm: Vec<usize> = (0..len).collect();
        for i in (1..len).rev() {
            let j = (rng.next_u32() as usize) % (i + 1);
            perm.swap(i, j);
        }
        perm
    }

    /// 应用排列
    fn apply_permutation(blocks: &[Vec<u8>], permutation: &[usize]) -> Vec<Vec<u8>> {
        let mut shuffled = Vec::with_capacity(blocks.len());
        for &idx in permutation {
            shuffled.push(blocks[idx].clone());
        }
        shuffled
    }

    /// 反转排列
    fn reverse_permutation(mut blocks: Vec<Vec<u8>>, permutation: &[usize]) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        if blocks.len() != permutation.len() {
            return Err("块数量与排列长度不匹配".into());
        }
        let mut restored = vec![Vec::new(); blocks.len()];
        for (i, &pos) in permutation.iter().enumerate() {
            if pos >= blocks.len() {
                return Err("排列索引越界".into());
            }
            std::mem::swap(&mut restored[pos], &mut blocks[i]);
        }
        Ok(restored)
    }

    /// 添加填充块：在随机位置插入随机块
    fn add_padding(mut blocks: Vec<Vec<u8>>, rng: &mut ChaCha20Rng) -> (Vec<Vec<u8>>, Vec<usize>) {
        let mut padding_positions = Vec::new();
        let mut i = 0;
        while i < blocks.len() {
            // 以1/3的概率在当前位置之前插入一个填充块
            if rng.next_u32() % 3 == 0 {
                let mut pad = vec![0u8; BLOCK_SIZE];
                rng.fill_bytes(&mut pad);
                blocks.insert(i, pad);
                padding_positions.push(i);
                i += 1; // 跳过刚插入的块
            }
            i += 1;
        }
        (blocks, padding_positions)
    }

    /// 移除填充块
    fn remove_padding(mut blocks: Vec<Vec<u8>>, padding_positions: &[usize]) -> Result<Vec<Vec<u8>>, Box<dyn Error>> {
        // 按降序排序位置，以便从后向前移除，避免索引变化
        let mut sorted_positions = padding_positions.to_vec();
        sorted_positions.sort_by(|a, b| b.cmp(a));
        for &pos in &sorted_positions {
            if pos >= blocks.len() {
                return Err("填充位置越界".into());
            }
            blocks.remove(pos);
        }
        Ok(blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generation() {
        let service = CryptoService::generate();
        let hex_key = service.private_key_hex();
        assert_eq!(hex_key.len(), 64); // 32字节 = 64十六进制字符
    }

    #[test]
    fn test_from_hex() {
        let key_hex = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
        let service = CryptoService::from_hex(key_hex).unwrap();
        assert_eq!(service.private_key_hex(), key_hex);
    }

    #[test]
    fn test_aes_encrypt_decrypt() {
        let service = CryptoService::generate();
        let plaintext = b"Hello, world!";
        let ciphertext = service.aes_encrypt(plaintext).unwrap();
        let decrypted = service.aes_decrypt(&ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_full_encrypt_decrypt() {
        let service = CryptoService::generate();
        let plaintext = b"This is a secret message that needs encryption.";
        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_different_keys() {
        let service1 = CryptoService::generate();
        let service2 = CryptoService::generate();
        let plaintext = b"Test";
        let encrypted = service1.encrypt(plaintext).unwrap();
        // 使用不同的密钥解密应该失败或得到错误结果
        let result = service2.decrypt(&encrypted);
        assert!(result.is_err());
    }
}