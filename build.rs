use std::process::Command;
use std::fs;

fn main() {
  let out_dir = "target/release/dfhooks_dfint.dll";
  let df_dir = r"D:\SteamLibrary\steamapps\common\Dwarf Fortress\dfhooks_dfint.dll";

  // Only copy if build succeeded
  if std::path::Path::new(out_dir).exists() {
    let _ = fs::copy(out_dir, df_dir);
    println!("cargo:warning=Copied dfhooks_dfint.dll to Dwarf Fortress folder");
  }
}
