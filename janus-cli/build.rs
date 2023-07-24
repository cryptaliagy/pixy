use clap::CommandFactory;

#[path = "src/cli.rs"]
mod cli;

/// Writes a manfile to the `target/man` directory. The name of the file
/// is dependent on the package name as set by cargo.
fn main() -> std::io::Result<()> {
    let current_dir = std::env::current_dir().map_err(|_| std::io::ErrorKind::NotFound)?;

    println!("current dir: {:?}", current_dir);
    println!("cargo:rerun-if-changed=src/cli.rs");
    println!("cargo:rerun-if-changed=build.rs");
    let mut out_dir = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").map_err(|_| std::io::ErrorKind::NotFound)?,
    );

    out_dir.pop();

    let pkg_name = std::env::var("CARGO_PKG_NAME").map_err(|_| std::io::ErrorKind::NotFound)?;
    let cmd = cli::Cli::command();

    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    let res = man.render(&mut buffer);

    if res.is_err() {
        print!("Error rendering manpage: {:?}", res)
    }

    let target_dir = out_dir.join("target").join("man");

    let create_dir_res = std::fs::create_dir(&target_dir);

    if let Err(create_error) = create_dir_res {
        match create_error.kind() {
            std::io::ErrorKind::AlreadyExists => {
                println!("Manpage dir already exists, continuing")
            }
            _ => {
                print!(
                    "Error creating manpage dir {:?}: {:?}",
                    &target_dir, create_error
                );
                return Err(create_error);
            }
        }
    }

    let manfile = format!("{}.1", pkg_name);

    let write_res = std::fs::write(target_dir.join(manfile), &buffer);

    if write_res.is_err() {
        print!("Error writing manpage: {:?}", write_res)
    }

    write_res
}
