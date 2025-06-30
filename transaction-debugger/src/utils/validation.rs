// Validation utilities

#[allow(dead_code)]
pub struct Validator;

#[allow(dead_code)]
impl Validator {
    // Validate Solana transaction signature format
    pub fn is_valid_signature(signature: &str) -> bool {
        // Solana signatures are typically 88 characters long
        if signature.len() != 88 {
            return false;
        }
        
        // Check if it contains only valid base58 characters
        signature.chars().all(|c| {
            c.is_ascii_alphanumeric() && 
            !matches!(c, '0' | 'O' | 'I' | 'l')
        })
    }
    
    // Validate RPC URL format
    pub fn is_valid_rpc_url(url: &str) -> bool {
        if url.is_empty() {
            return false;
        }
        
        // Basic URL validation
        url.starts_with("http://") || url.starts_with("https://")
    }
    
    // Validate Solana public key format
    pub fn is_valid_pubkey(pubkey: &str) -> bool {
        // Solana pubkeys are typically 44 characters long
        if pubkey.len() != 44 {
            return false;
        }
        
        // Check if it contains only valid base58 characters
        pubkey.chars().all(|c| {
            c.is_ascii_alphanumeric() && 
            !matches!(c, '0' | 'O' | 'I' | 'l')
        })
    }
}
