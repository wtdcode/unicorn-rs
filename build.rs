use std::result::Result;
use std::{env, process::Command};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let profile = env::var("PROFILE").unwrap();
    let mut version = String::from("master");
    if let Result::Ok(version_env) = env::var("UNICORN_VERISON") {
        version = version_env;
    }

    let unicorn_dir = format!("{}/unicorn_git", out_dir);

    Command::new("git")
        .arg("clone")
        .arg("https://github.com/unicorn-engine/unicorn")
        .arg("-b")
        .arg(version)
        .arg(&unicorn_dir)
        .output()
        .expect("Fail to clone Unicorn repository.");

    if env::consts::OS == "windows" {
        // Windows
        let mut platform = "x64";
        let mut conf = "Release";
        if std::mem::size_of::<usize>() == 4 {
            platform = "Win32";
        }
        if profile == "debug" {
            conf = "Debug";
        }

        Command::new("msbuild")
            .current_dir(format!("{}/msvc", &unicorn_dir))
            .arg("unicorn.sln")
            .arg("-m")
            .arg("-p:Platform".to_owned() + platform)
            .arg("-p:Configuration".to_owned() + conf)
            .output()
            .expect("Fail to build unicorn on Win32.");
        println!(
            "cargo:rustc-link-lib=static={}/msvc/{}/{}/unicorn.lib",
            unicorn_dir, platform, conf
        );
    } else {
        // Most Unix-like systems
        let mut cmd = Command::new("sh");
        cmd.current_dir(&unicorn_dir).arg("make.sh");

        if profile == "debug" {
            cmd.env("UNICORN_DEBUG", "yes");
        }

        cmd.output().expect("Fail to build unicorn on *nix.");

        println!("cargo:rustc-link-lib=unicorn");
        println!("cargo:rustc-link-search={}", unicorn_dir);
    }
}
