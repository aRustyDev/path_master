fn main() -> std::io::Result<()> {
    let out_dir = std::path::PathBuf::from("./docs/man/");

    // Ensure the directory exists. If it doesn't, create it and any necessary parent directories.
    std::fs::create_dir_all(&out_dir)?;

    let cmd = clap::Command::new("path_master")
        .arg(clap::arg!(-n --name <NAME>))
        .arg(clap::arg!(-c --count <NUM>));

    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    // Join the filename to the output directory path and write the buffer.
    std::fs::write(out_dir.join("path_master.1"), buffer)?;

    Ok(())
}
