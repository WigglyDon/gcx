// SPDX-License-Identifier: MIT
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: ntx-migrate <protected-export-root> <private-target-parent>; never point this at live device data".into());
    }
    let outcome = ntx::migration::import_export(
        std::path::Path::new(&args[0]),
        std::path::Path::new(&args[1]),
    )?;
    println!(
        "{outcome:?}; source retained; scoped media access and legacy scene recovery still require explicit handling"
    );
    Ok(())
}
