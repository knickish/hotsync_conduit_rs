# hotsync_conduit_rs
## create and install native hotsync conduits

### prerequisites
* HotSync Manager
* Cargo
* Native Windows and 32-Bit Windows Rust toolchains (`rustup target add i686-pc-windows-msvc`)

### use
Use the `Conduit` type to define and install a conduit:
```rust
ConduitInstallerBuilder::new_with_creator(
        ['T', 'e', 's', 't'],
        CString::new("testing.dll")?,
    )?
    .with_title(CString::new("Conduit Title")?)
    .build()?
    .install()?;
```