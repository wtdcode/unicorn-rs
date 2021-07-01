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
        .arg("git@github.com:wtdcode/unicorn2.git") // TODO: Change it to the official repo.
        .arg("-b")
        .arg(version)
        .arg(&unicorn_dir)
        .output()
        .expect("Fail to clone Unicorn repository.");

    let unicorn_dir = format!("{}/unicorn2", unicorn_dir); // TODO: Remove this line after release.

    // We don't use TARGET since we can't cross-build.
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
        let mut cmd = Command::new("cmake");
        cmd.current_dir(&unicorn_dir)
            .arg("-B")
            .arg("rust_build")
            .arg("-DUNICORN_BUILD_SHARED=off");

        if profile == "debug" {
            cmd.arg("-DCMAKE_BUILD_TYPE=Debug");
        } else {
            cmd.arg("-DCMAKE_BUILD_TYPE=Release");
        }

        cmd.output()
            .expect("Fail to create build directory on *nix.");

        Command::new("make")
            .current_dir(format!("{}/rust_build", &unicorn_dir))
            .arg("-j6")
            .output()
            .expect("Fail to build unicorn on *nix.");

        println!("cargo:rustc-link-lib=unicorn");
        for arch in [
            "x86_64", "arm", "armeb", "aarch64", "aarch64eb", "riscv32", "riscv64", "mips",
            "mipsel", "mips64", "mips64el", "sparc", "sparc64", "m68k", "ppc", "ppc64",
        ]
        .iter()
        {
            println!("cargo:rustc-link-lib={}-softmmu", arch);
        }
        println!("cargo:rustc-link-search={}/rust_build", unicorn_dir);
    }
}
