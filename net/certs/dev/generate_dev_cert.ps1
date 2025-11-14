# Generate self-signed development certificates for TLS testing
# PowerShell script for Windows

# Check if OpenSSL is available
$openssl = Get-Command openssl -ErrorAction SilentlyContinue

if ($openssl) {
    Write-Host "Using OpenSSL to generate certificates..."
    & openssl req -x509 -newkey rsa:4096 -nodes `
        -keyout dev-key.pem `
        -out dev-cert.pem `
        -days 365 `
        -subj "/CN=localhost"
    
    Write-Host "Generated dev-cert.pem and dev-key.pem for localhost"
    Write-Host "These certificates are for DEVELOPMENT ONLY" -ForegroundColor Yellow
} else {
    Write-Host "OpenSSL not found. Please install OpenSSL or use WSL to run generate_dev_cert.sh" -ForegroundColor Red
    Write-Host ""
    Write-Host "Alternative: Use New-SelfSignedCertificate (requires conversion to PEM):"
    Write-Host '  $cert = New-SelfSignedCertificate -DnsName "localhost" -CertStoreLocation "cert:\CurrentUser\My"'
    Write-Host '  Then export and convert to PEM format'
}
