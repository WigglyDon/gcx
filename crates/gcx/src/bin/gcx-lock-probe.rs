// SPDX-License-Identifier: MIT
//! Bounded stock-Android qualification helper, using the product's actual lock.
use gcx::ProfileLock;
use std::{
    io::Write,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if !(2..=3).contains(&args.len()) {
        return Err("usage: gcx-lock-probe <expect-busy|expect-free|hold|watch-release> <existing-profile> [seconds]".into());
    }
    let profile = PathBuf::from(&args[1]).canonicalize()?;
    if args[0] == "watch-release" && args.len() == 3 {
        let seconds: u64 = args[2].to_str().ok_or("invalid duration")?.parse()?;
        if !(1..=20).contains(&seconds) {
            return Err("watch duration must be 1..20 seconds".into());
        }
        let deadline = Instant::now() + Duration::from_secs(seconds);
        let mut samples = 0;
        loop {
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
            match ProfileLock::acquire(&profile) {
                Ok(guard) => {
                    drop(guard);
                    if samples == 0 {
                        return Err("expected an already-held runtime writer".into());
                    }
                    println!("RELEASED {timestamp} busy_samples={samples}");
                    return Ok(());
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    if samples == 0 {
                        println!("READY: runtime writer held");
                    }
                    println!("BUSY {timestamp}");
                    std::io::stdout().flush()?;
                    samples += 1;
                }
                Err(error) => return Err(error.into()),
            }
            if Instant::now() >= deadline {
                return Err("writer did not release before bounded deadline".into());
            }
            std::thread::sleep(Duration::from_millis(2));
        }
    }
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
