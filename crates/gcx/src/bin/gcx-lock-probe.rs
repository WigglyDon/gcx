// SPDX-License-Identifier: MIT
//! Bounded stock-Android qualification helper, using the product's actual lock.
use gcx::ProfileLock;
use std::{io::Write, path::PathBuf, time::Duration};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !(2..=3).contains(&args.len()) {
        return Err("usage: gcx-lock-probe <expect-busy|expect-free|hold> <existing-profile> [1..30 seconds]".into());
    }
    let profile = PathBuf::from(&args[1]).canonicalize()?;
    let result = ProfileLock::acquire(&profile);
    if args[0] == "expect-busy" && args.len() == 2 {
        if !matches!(result, Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock) {
            return Err("FAIL: another writer was not excluded".into());
        }
        println!("PASS: product profile lock excluded this process");
    } else if args[0] == "expect-free" && args.len() == 2 {
        let _owner = result?;
        println!("PASS: product profile lock acquired after release");
    } else if args[0] == "hold" && args.len() == 3 {
        let seconds: u64 = args[2].to_str().ok_or("invalid duration")?.parse()?;
        if !(1..=30).contains(&seconds) {
            return Err("hold duration must be 1..30 seconds".into());
        }
        let _owner = result?;
        println!("READY: holding product profile lock");
        std::io::stdout().flush()?;
        std::thread::sleep(Duration::from_secs(seconds));
        println!("RELEASED: bounded hold completed");
    } else {
        return Err("invalid lock probe command".into());
    }
    Ok(())
}
