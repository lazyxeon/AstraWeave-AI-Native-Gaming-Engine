# Development TLS Certificates

This directory contains self-signed certificates for local development and testing.

## Generating Certificates

### Option 1: Using OpenSSL (Recommended)

If you have OpenSSL installed:
```bash
openssl req -x509 -newkey rsa:4096 -nodes \
  -keyout dev-key.pem \
  -out dev-cert.pem \
  -days 365 \
  -subj "/CN=localhost"
```

Or run the provided script:
```bash
# Linux/macOS/WSL
bash generate_dev_cert.sh

# Windows with OpenSSL
powershell -ExecutionPolicy Bypass -File generate_dev_cert.ps1
```

### Option 2: Using WSL (Windows)

If OpenSSL is not installed on Windows but you have WSL:
```bash
wsl bash generate_dev_cert.sh
```

### Option 3: Install OpenSSL

**Windows:**
- Download from: https://slproweb.com/products/Win32OpenSSL.html
- Or install via chocolatey: `choco install openssl`

**macOS:**
```bash
brew install openssl
```

**Linux:**
```bash
sudo apt install openssl  # Debian/Ubuntu
sudo yum install openssl  # RHEL/CentOS
```

## Files

- `dev-cert.pem` - Public certificate
- `dev-key.pem` - Private key
- `generate_dev_cert.sh` - Bash script for certificate generation
- `generate_dev_cert.ps1` - PowerShell script for certificate generation

## Security Warning

**THESE CERTIFICATES ARE FOR DEVELOPMENT ONLY!**

- Self-signed certificates are not trusted by browsers
- Do NOT use these certificates in production
- For production, use certificates from a trusted CA (Let's Encrypt, DigiCert, etc.)
