use base64::{Engine, engine::general_purpose::STANDARD as B64};
use cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use rsa::{Pkcs1v15Encrypt, RsaPublicKey, pkcs8::DecodePublicKey};
use serde::Serialize;

const RSA_PUBLIC_KEY_PEM: &str = "-----BEGIN PUBLIC KEY-----\n\
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDA8iMH5c02LilrsERw9t6Pv5Nc\n\
4k6Pz1EaDicBMpdpxKduSZu5OANqUq8er4GM95omAGIOPOh+Nx0spthYA2BqGz+l\n\
6HRkPJ7S236FZz73In/KVuLnwI8JJ2CbuJap8kvheCCZpmAWpb/cPx/3Vr/J6I17\n\
XcW+ML9FoCI6AOvOzwIDAQAB\n\
-----END PUBLIC KEY-----";

#[derive(Debug, Clone, Serialize)]
pub struct CosyHeaders {
    pub authorization: String,
    pub cosy_user: String,
    pub cosy_key: String,
    pub cosy_date: String,
    pub request_id: String,
}

pub fn build_cosy_headers(
    token: &str,
    user_id: &str,
    name: &str,
    email: &str,
    request_path: &str,
    request_body: &str,
) -> Result<CosyHeaders, String> {
    // 1. 生成随机 AES key
    let uuid_str = uuid::Uuid::new_v4().to_string().replace('-', "");
    let aes_key = &uuid_str[..16];

    // 2. AES-128-CBC 加密用户信息
    let user_info = serde_json::json!({
        "uid": user_id,
        "aid": "",
        "name": name,
        "email": email,
        "security_oauth_token": token
    });
    let encrypted_info = cosy_aes_encrypt(&user_info.to_string(), aes_key)?;

    // 3. RSA 加密 AES key
    let rsa_encrypted_key = cosy_rsa_encrypt(aes_key.as_bytes())?;

    // 4. 构建 payload
    let request_id = uuid::Uuid::new_v4().to_string().replace('-', "");
    let timestamp = chrono::Utc::now().timestamp().to_string();
    let payload = serde_json::json!({
        "version": "v1",
        "requestId": &request_id,
        "info": encrypted_info,
        "cosyVersion": "1.2.2",
        "ideVersion": ""
    });
    let payload_b64 = B64.encode(payload.to_string().as_bytes());

    // 5. MD5 签名
    let sig_input = format!("{}\n{}\n{}\n{}\n{}",
        payload_b64, rsa_encrypted_key, timestamp, request_body, request_path);
    let signature = cosy_md5(&sig_input);

    Ok(CosyHeaders {
        authorization: format!("Bearer COSY.{}.{}", payload_b64, signature),
        cosy_user: user_id.to_string(),
        cosy_key: rsa_encrypted_key,
        cosy_date: timestamp,
        request_id,
    })
}

fn cosy_aes_encrypt(plaintext: &str, key: &str) -> Result<String, String> {
    type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
    let key_bytes = key.as_bytes();
    // Cosy 协议约定：IV 复用 AES key 本身（非标准密码学实践，但为保持与 Qoder 服务端兼容必须如此）
    let iv = &key_bytes[..16];
    let enc = Aes128CbcEnc::new_from_slices(key_bytes, iv)
        .map_err(|e| format!("AES init failed: {}", e))?;
    let pt = plaintext.as_bytes();
    let block_size = 16;
    let buf_len = pt.len() + block_size;
    let mut buf = vec![0u8; buf_len];
    buf[..pt.len()].copy_from_slice(pt);
    let ct = enc.encrypt_padded_mut::<Pkcs7>(&mut buf, pt.len())
        .map_err(|e| format!("AES encrypt failed: {}", e))?;
    Ok(B64.encode(ct))
}

fn cosy_rsa_encrypt(data: &[u8]) -> Result<String, String> {
    let pub_key = RsaPublicKey::from_public_key_pem(RSA_PUBLIC_KEY_PEM)
        .map_err(|e| format!("RSA key parse failed: {}", e))?;
    let mut rng = rsa::rand_core::OsRng;
    let encrypted = pub_key.encrypt(&mut rng, Pkcs1v15Encrypt, data)
        .map_err(|e| format!("RSA encrypt failed: {}", e))?;
    Ok(B64.encode(&encrypted))
}

fn cosy_md5(input: &str) -> String {
    use md5::Digest;
    let mut hasher = md5::Md5::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}
