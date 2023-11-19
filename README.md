# hotsync_conduit_rs
## create and install native hotsync conduits

### prerequisites
* HotSync Manager
* Cargo
* 32-Bit Windows Rust toolchains (`rustup target add i686-pc-windows-msvc`)

### use
Add the following to the `.cargo/config` for your crate: 
```
[build]
target = "i686-pc-windows-msvc"
```
From a cdylib target, export a function named 'OpenConduit' with the signature shown below, using the `ConduitBuilder` type to implement the sync functionality for your conduit. 

```rust
#[no_mangle]
pub unsafe extern "cdecl" fn OpenConduit(
    _: *const c_void,
    sync_props: *const CSyncProperties,
) -> c_long {
    let database: PalmDatabase::<PdbDatabase> = fn_that_generates_your_db();
    let conduit =
        ConduitBuilder::new_with_name_creator(
            CString::new("example_conduit").unwrap(), 
            [b'T', b'e', b's', b't']
        )
        .overwrite_db(ConduitDBSource::Static(
            CString::new("ExampleContentDB").unwrap(),
            [b'D', b'A', b'T', b'A'],
            database,
        ))
        .build();

    match conduit.sync() {
        Ok(_) => 0,
        Err(_) => -1,
    }
}
```
Use the `ConduitInstallation` and `ConduitManager` types to define and install a conduit with the same CreatorID used in your on-device application:
```rust
let builder = ConduitInstallation::new_with_creator(
    ['T', 'e', 's', 't'],
    CString::new("example_conduit.dll").unwrap(),
)
.unwrap()
.with_title(CString::new("Example").unwrap());

ConduitManager::initialize()
    .unwrap()
    .reinstall(builder, None)
    .unwrap();
```

Finally, move the `*.dll` you made previously into your hotsync folder. This can also be done programatically in the previous step. Your conduit is now enabled, and will run during each hotsync of a device with the relevant application installed.