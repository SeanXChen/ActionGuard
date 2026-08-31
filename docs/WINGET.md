# Publishing ActionGuard to winget

Windows users install apps with `winget`. This document is the checklist for
getting ActionGuard into the official **winget-pkgs** community repository.

## Why not `curl | sh`

ActionGuard is a security product. Telling a user to pipe a remote shell
script into their shell is exactly the trust pattern this product exists to
prevent. The supported path is always:

```
Download → Verify checksum → Install
```

## 1. Cut a release first

`winget` validates the installer URL and `InstallerSha256` against a **real**
GitHub Release, so the installer must exist and be final before submission.

1. Tag and push: `git tag v0.3.0 && git push origin v0.3.0`
2. The Release workflow builds installers (MSI + NSIS on Windows) and creates
   a **draft** release with a `SHA256SUMS` file.
3. Manually smoke-test the NSIS installer on a clean VM, then publish the
   draft release.

## 2. Fill in the manifest hash

Edit `scripts/winget/SeanXChen.ActionGuard.installer.yaml` and set:

```yaml
InstallerSha256: <value>
```

```powershell
# from the download folder
Get-FileHash .\ActionGuard_0.3.0_x64-setup.exe -Algorithm SHA256
```

## 3. Validate locally

```powershell
winget install --manifest .\scripts\winget
```

## 4. Submit to winget-pkgs

1. Fork https://github.com/microsoft/winget-pkgs
2. Copy `scripts/winget/` as
   `manifests/s/Se/SeanXChen.ActionGuard/0.3.0/`
3. Open a PR titled `New version: SeanXChen.ActionGuard version 0.3.0`
4. The validation bot checks the hash and URL; a human maintainer approves.

## 5. Installer signing (roadmap)

Un-signed Windows executables trigger the SmartScreen "Windows protected your
PC" warning. This is expected on the first release. The roadmap:

- **Short term:** document the warning in the README; the `SHA256SUMS` file in
  every release already gives users a way to verify the binary they got is the
  binary that was built.
- **Long term:** sign with a code-signing certificate (or Azure Trusted
  Signing) and store the signing secret in GitHub Actions secrets; add a
  `sign` step to `.github/workflows/release.yml` before upload.
