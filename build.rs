#[cfg(feature = "manpages")]
include!("src/cli.rs");

#[cfg(feature = "manpages")]
fn build_manpages(outdir: &std::path::Path) -> Result<(), std::io::Error> {
    let app = cli();

    let file = std::path::Path::new(&outdir).join("example.1");
    let mut file = std::fs::File::create(&file)?;

    clap_mangen::Man::new(app).render(&mut file)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "manpages")]
    {
        println!("cargo:rerun-if-changed=src/cli.rs");
        println!("cargo:rerun-if-changed=man");

        let outdir = match std::env::var_os("OUT_DIR") {
            None => return Ok(()),
            Some(outdir) => outdir,
        };

        // Create `target/assets/` folder.
        let out_path = std::path::PathBuf::from(outdir);
        let mut path = out_path.ancestors().nth(4).unwrap().to_owned();
        path.push("assets");
        std::fs::create_dir_all(&path).unwrap();

        build_manpages(&path)?;
    }

    Ok(())
}
