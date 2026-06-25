use std::path::{Path, PathBuf};
use std::fs;
use base64::Engine;

pub struct CertManager {
    ca_dir: PathBuf,
}

impl CertManager {
    pub fn new(data_dir: &Path) -> Self {
        let ca_dir = data_dir.join("proxy-ca");
        let _ = fs::create_dir_all(&ca_dir);
        Self { ca_dir }
    }

    pub fn ca_key_path(&self) -> PathBuf {
        self.ca_dir.join("ca.key")
    }

    pub fn ca_cert_path(&self) -> PathBuf {
        self.ca_dir.join("ca.crt")
    }

    pub fn install_ca(&self) -> Result<(), String> {
        let cert_path = self.ca_cert_path();
        if !cert_path.exists() {
            return Err("CA 证书不存在，请先启动代理生成证书".to_string());
        }

        // 读取 PEM 证书并转换为 DER
        let pem_data = fs::read_to_string(&cert_path)
            .map_err(|e| format!("读取证书文件失败: {}", e))?;
        let der = pem_to_der(&pem_data)?;

        #[cfg(target_os = "windows")]
        {
            win_cert::install_cert_silent(&der)
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = der;
            Err("当前平台不支持自动安装证书".to_string())
        }
    }

    pub fn uninstall_ca(&self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            win_cert::uninstall_cert_silent("Qoder Proxy CA")
        }
        #[cfg(not(target_os = "windows"))]
        {
            Err("当前平台不支持自动卸载证书".to_string())
        }
    }

    pub fn is_installed(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            win_cert::is_cert_installed("Qoder Proxy CA")
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }
}

/// PEM 格式转 DER 二进制
fn pem_to_der(pem: &str) -> Result<Vec<u8>, String> {
    let pem = pem.trim();
    let b64: String = pem
        .lines()
        .filter(|l| !l.starts_with("-----"))
        .collect::<Vec<_>>()
        .join("");
    base64::engine::general_purpose::STANDARD
        .decode(&b64)
        .map_err(|e| format!("Base64 解码失败: {}", e))
}

#[cfg(target_os = "windows")]
mod win_cert {
    use windows::Win32::Security::Cryptography::*;

    /// 静默安装 CA 证书到当前用户的 Root 证书库
    pub fn install_cert_silent(cert_der: &[u8]) -> Result<(), String> {
        unsafe {
            let store_name = windows::core::w!("Root");
            let store = CertOpenStore(
                CERT_STORE_PROV_SYSTEM_W,
                CERT_QUERY_ENCODING_TYPE(0),
                None,
                CERT_OPEN_STORE_FLAGS(CERT_SYSTEM_STORE_CURRENT_USER),
                Some(store_name.as_ptr() as *const _),
            )
            .map_err(|e| format!("打开证书库失败: {}", e))?;

            // 创建证书上下文
            let cert_ctx = CertCreateCertificateContext(
                X509_ASN_ENCODING | PKCS_7_ASN_ENCODING,
                cert_der,
            );
            if cert_ctx.is_null() {
                let _ = CertCloseStore(store, 0);
                return Err("创建证书上下文失败".to_string());
            }

            // 添加到证书库（如果已存在则替换）
            let result = CertAddCertificateContextToStore(
                store,
                cert_ctx,
                CERT_STORE_ADD_REPLACE_EXISTING,
                None,
            );

            let _ = CertFreeCertificateContext(Some(cert_ctx));
            let _ = CertCloseStore(store, 0);

            result.map_err(|e| format!("添加证书到证书库失败: {}", e))
        }
    }

    /// 静默卸载 CA 证书
    pub fn uninstall_cert_silent(subject_name: &str) -> Result<(), String> {
        unsafe {
            let store_name = windows::core::w!("Root");
            let store = CertOpenStore(
                CERT_STORE_PROV_SYSTEM_W,
                CERT_QUERY_ENCODING_TYPE(0),
                None,
                CERT_OPEN_STORE_FLAGS(CERT_SYSTEM_STORE_CURRENT_USER),
                Some(store_name.as_ptr() as *const _),
            )
            .map_err(|e| format!("打开证书库失败: {}", e))?;

            // 遍历查找匹配的证书并删除
            let mut cert_ctx = CertEnumCertificatesInStore(store, None);
            while !cert_ctx.is_null() {
                let mut name_buf = vec![0u8; 256];
                let name_len = CertGetNameStringA(
                    cert_ctx,
                    CERT_NAME_SIMPLE_DISPLAY_TYPE,
                    0,
                    None,
                    Some(&mut name_buf),
                );
                if name_len > 0 {
                    let name =
                        String::from_utf8_lossy(&name_buf[..name_len as usize - 1]);
                    if name.contains(subject_name) {
                        // 复制上下文后删除（CertDeleteCertificateFromStore 会释放传入的 context）
                        let dup =
                            CertDuplicateCertificateContext(Some(cert_ctx as *const _));
                        let _ = CertDeleteCertificateFromStore(dup);
                    }
                }
                cert_ctx = CertEnumCertificatesInStore(store, Some(cert_ctx));
            }

            let _ = CertCloseStore(store, 0);
            Ok(())
        }
    }

    /// 检测证书是否已安装
    pub fn is_cert_installed(subject_name: &str) -> bool {
        unsafe {
            let store_name = windows::core::w!("Root");
            let store = match CertOpenStore(
                CERT_STORE_PROV_SYSTEM_W,
                CERT_QUERY_ENCODING_TYPE(0),
                None,
                CERT_OPEN_STORE_FLAGS(CERT_SYSTEM_STORE_CURRENT_USER),
                Some(store_name.as_ptr() as *const _),
            ) {
                Ok(s) => s,
                Err(_) => return false,
            };

            let mut found = false;
            let mut cert_ctx = CertEnumCertificatesInStore(store, None);
            while !cert_ctx.is_null() {
                let mut name_buf = vec![0u8; 256];
                let name_len = CertGetNameStringA(
                    cert_ctx,
                    CERT_NAME_SIMPLE_DISPLAY_TYPE,
                    0,
                    None,
                    Some(&mut name_buf),
                );
                if name_len > 0 {
                    let name =
                        String::from_utf8_lossy(&name_buf[..name_len as usize - 1]);
                    if name.contains(subject_name) {
                        found = true;
                        let _ = CertFreeCertificateContext(Some(cert_ctx));
                        break;
                    }
                }
                cert_ctx = CertEnumCertificatesInStore(store, Some(cert_ctx));
            }

            let _ = CertCloseStore(store, 0);
            found
        }
    }
}
